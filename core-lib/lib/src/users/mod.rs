mod data;
#[cfg(not(feature = "i-am-core"))]
mod db;
#[cfg(feature = "i-am-core")]
pub mod db;
mod timeout;

pub use data::{Rights, Session, Token, User};
pub use timeout::Timeout;

use crate::ffi;

use std::fmt;
use std::mem::MaybeUninit;

use chuchi::{header::RequestHeader, Resource};

use chuchi_postgres::Database;

mod helpers {
	use super::*;

	pub fn get_token(header: &RequestHeader) -> Option<Token> {
		header.value("auth-token").and_then(|t| t.parse().ok())
	}

	pub fn get_token_from_cookie(header: &RequestHeader) -> Option<Token> {
		header
			.value("cookie")
			.and_then(|v| v.trim().strip_prefix("data-token="))
			.and_then(|t| t.trim().parse().ok())
	}
}
#[cfg(feature = "i-am-core")]
pub use helpers::{get_token, get_token_from_cookie};
#[cfg(not(feature = "i-am-core"))]
pub use helpers::{get_token, get_token_from_cookie};

#[derive(Debug)]
pub enum Error {
	MissingAuthToken,
	MissingDataToken,
	InvalidAuthToken,
	InvalidDataToken,
	InvalidUser,
	Db(chuchi_postgres::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {}

#[derive(Resource)]
pub struct Users {
	db: db::Users,
	sessions: Sessions,
}

impl Users {
	pub async fn new(db: &Database, sessions: Sessions) -> Self {
		Self {
			db: db::Users::new(db).await,
			sessions,
		}
	}

	pub async fn sess_user_from_req(
		&self,
		header: &RequestHeader,
	) -> Result<(Session, User), Error> {
		let token = get_token(header).ok_or(Error::MissingAuthToken)?;

		self.sess_user_from_token(&token).await
	}

	pub async fn sess_user_from_token(
		&self,
		token: &Token,
	) -> Result<(Session, User), Error> {
		let sess = self
			.sessions
			.by_token(&token)
			.ok_or(Error::InvalidAuthToken)?;
		let user = self
			.db
			.by_id(&sess.user_id)
			.await
			.map_err(Error::Db)?
			.ok_or(Error::InvalidUser)?;

		Ok((sess, user))
	}

	pub async fn sess_user_from_cookie(
		&self,
		header: &RequestHeader,
	) -> Result<(Session, User), Error> {
		let token: Token =
			get_token_from_cookie(header).ok_or(Error::MissingDataToken)?;

		let sess = self
			.sessions
			.by_data_token(&token)
			.ok_or(Error::InvalidDataToken)?;
		let user = self
			.db
			.by_id(&sess.user_id)
			.await
			.map_err(Error::Db)?
			.ok_or(Error::InvalidUser)?;

		Ok((sess, user))
	}
}

pub struct Sessions {
	inner: ffi::c_sessions,
}

impl Sessions {
	pub fn new(inner: ffi::c_sessions) -> Self {
		Self { inner }
	}

	pub fn by_token(&self, token: &Token) -> Option<Session> {
		let mut sess = MaybeUninit::uninit();
		let token = ffi::c_token::from_token(token.clone());
		let some =
			(self.inner.by_token)(self.inner.ctx, token, sess.as_mut_ptr());

		if some {
			Some(Session::from_c(unsafe { sess.assume_init() }))
		} else {
			None
		}
	}

	pub fn by_data_token(&self, token: &Token) -> Option<Session> {
		let mut sess = MaybeUninit::uninit();
		let token = ffi::c_token::from_token(token.clone());
		let some = (self.inner.by_data_token)(
			self.inner.ctx,
			token,
			sess.as_mut_ptr(),
		);

		if some {
			Some(Session::from_c(unsafe { sess.assume_init() }))
		} else {
			None
		}
	}
}

impl Drop for Sessions {
	fn drop(&mut self) {
		(self.inner.free)(self.inner.ctx as *mut _);
	}
}

unsafe impl Send for Sessions {}
unsafe impl Sync for Sessions {}
