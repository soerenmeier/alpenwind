use super::{Rights, Session, Timeout, Token, User};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bcrypt::{hash, verify};
use core_lib::ffi;
use core_lib::users::db::UnsafeUser;

use chuchi::Resource;
use chuchi_postgres::json::Json;
use chuchi_postgres::table::TableOwned;
use chuchi_postgres::{filter, row, try2, whr, Error};
use chuchi_postgres::{Database, Result, UniqueId};

#[derive(Debug, Clone, Resource)]
pub struct Users {
	table: TableOwned<UnsafeUser>,
	sessions: Sessions,
}

impl Users {
	pub async fn new(db: &Database) -> Self {
		Self {
			table: db.table_owned("users").create().await,
			sessions: Sessions::new(),
		}
	}

	#[allow(dead_code)]
	pub async fn all(&self) -> Result<Vec<User>> {
		let users = self.table.find_all().await?;

		Ok(users.into_iter().map(Into::into).collect())
	}

	pub async fn by_id(&self, id: &UniqueId) -> Result<Option<User>> {
		self.table
			.find_one(filter!(id))
			.await
			.map(|opt| opt.map(Into::into))
	}

	pub async fn login(
		&self,
		username: &str,
		password: &str,
	) -> Result<Option<User>> {
		let username = &username;
		let user = try2!(self.table.find_one(filter!(username)).await?);

		let passwd_corr = verify(password, &user.password).unwrap_or(false);

		Ok(passwd_corr.then(|| user.into()))
	}

	pub async fn insert(
		&self,
		username: String,
		name: String,
		password: String,
		rights: Rights,
	) -> Result<User> {
		let user = UnsafeUser {
			id: UniqueId::new(),
			password: hash(&password, 12)
				.map_err(|e| Error::Unknown(e.into()))?,
			username,
			name,
			rights: Json(rights),
		};

		self.table.insert_one(&user).await?;

		Ok(user.into())
	}

	// the password will get hashed
	pub async fn update(
		&self,
		id: &UniqueId,
		name: &str,
		password: Option<&str>,
	) -> Result<()> {
		if let Some(password) = password {
			let password =
				hash(password, 12).map_err(|e| Error::Unknown(e.into()))?;
			self.table.update(row! { &name, &password }, whr!(id)).await
		} else {
			self.table.update(row! { &name }, whr!(id)).await
		}
	}

	pub fn sessions_cleanup(&self) {
		self.sessions.cleanup();
	}

	pub async fn by_sess_token(
		&self,
		token: &Token,
	) -> Result<Option<(Session, User)>> {
		let sess = try2!(self.session_by_token(token));
		let user = try2!(self.by_id(&sess.user_id).await?);

		Ok(Some((sess, user)))
	}

	pub fn session_by_token(&self, token: &Token) -> Option<Session> {
		self.sessions.find(token)
	}

	pub async fn by_data_token(
		&self,
		token: &Token,
	) -> Result<Option<(Session, User)>> {
		let sess = try2!(self.session_by_data_token(token));
		let user = try2!(self.by_id(&sess.user_id).await?);

		Ok(Some((sess, user)))
	}

	pub fn session_by_data_token(&self, token: &Token) -> Option<Session> {
		self.sessions.find_by_data(token)
	}

	pub fn session_insert(
		&self,
		user_id: UniqueId,
		timeout: Timeout,
	) -> Session {
		self.sessions.insert(user_id, timeout)
	}

	pub fn session_remove(&self, token: &Token) {
		self.sessions.remove(token);
	}

	pub fn to_sessions_c(&self) -> ffi::c_sessions {
		self.sessions.to_c()
	}
}

#[derive(Debug, Clone)]
struct Sessions {
	inner: Arc<RwLock<Inner>>,
}

impl Sessions {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(RwLock::new(Inner::new())),
		}
	}

	pub fn find(&self, token: &Token) -> Option<Session> {
		let reader = self.inner.read().unwrap();
		reader.get(token).map(Clone::clone)
	}

	pub fn find_by_data(&self, token: &Token) -> Option<Session> {
		let reader = self.inner.read().unwrap();
		reader.get_by_data(token).map(Clone::clone)
	}

	pub fn insert(&self, user_id: UniqueId, timeout: Timeout) -> Session {
		let mut writer = self.inner.write().unwrap();

		let sess = Session::new(timeout, user_id);

		writer.insert(sess.token.clone(), sess.clone());

		sess
	}

	pub fn remove(&self, token: &Token) {
		self.inner.write().unwrap().remove(token);
	}

	pub fn cleanup(&self) {
		self.inner.write().unwrap().cleanup();
	}

	fn into_ptr(self) -> *const u8 {
		Arc::into_raw(self.inner) as *const _
	}

	unsafe fn clone_from_ptr(ptr: *const u8) -> Self {
		Arc::increment_strong_count(ptr as *const Arc<RwLock<Inner>>);

		Self::from_ptr(ptr)
	}

	unsafe fn from_ptr(ptr: *const u8) -> Self {
		Self {
			inner: Arc::from_raw(ptr as *const _),
		}
	}

	fn to_c(&self) -> ffi::c_sessions {
		let me = self.clone();

		extern "C" fn by_token(
			ctx: *const u8,
			token: ffi::c_token,
			session: *mut ffi::c_session,
		) -> bool {
			let sessions = unsafe { Sessions::clone_from_ptr(ctx) };

			match sessions.find(&token.into_token()) {
				Some(sess) => {
					unsafe { session.write(sess.into_c()) };
					true
				}
				None => false,
			}
		}

		extern "C" fn by_data_token(
			ctx: *const u8,
			token: ffi::c_token,
			session: *mut ffi::c_session,
		) -> bool {
			let sessions = unsafe { Sessions::clone_from_ptr(ctx) };

			match sessions.find_by_data(&token.into_token()) {
				Some(sess) => {
					unsafe { session.write(sess.into_c()) };
					true
				}
				None => false,
			}
		}

		extern "C" fn free(ctx: *const u8) {
			drop(unsafe { Sessions::from_ptr(ctx) });
		}

		ffi::c_sessions {
			ctx: me.into_ptr(),
			by_token,
			by_data_token,
			free,
		}
	}
}

#[derive(Debug)]
struct Inner {
	inner: HashMap<Token, Session>,
	data: HashMap<Token, Token>,
}

impl Inner {
	fn new() -> Self {
		Self {
			inner: HashMap::new(),
			data: HashMap::new(),
		}
	}

	fn get(&self, token: &Token) -> Option<&Session> {
		self.inner.get(token).filter(|s| !s.did_timeout())
	}

	fn get_by_data(&self, token: &Token) -> Option<&Session> {
		self.data
			.get(token)
			.and_then(|t| self.inner.get(t))
			.filter(|s| !s.did_timeout())
	}

	fn insert(&mut self, token: Token, session: Session) {
		self.data.insert(session.data_token.clone(), token.clone());
		self.inner.insert(token, session);
	}

	fn remove(&mut self, token: &Token) {
		let sess = self.inner.remove(token);
		if let Some(sess) = sess {
			self.data.remove(&sess.data_token);
		}
	}

	fn cleanup(&mut self) {
		self.inner.retain(|_, s| {
			if s.did_timeout() {
				self.data.remove(&s.data_token);
				false
			} else {
				true
			}
		});
	}
}
