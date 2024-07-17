use chuchi::Resource;
use chuchi_postgres::Database;

use super::{db, Error, Session, Sessions, Token, User};

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

	pub async fn sess_user_from_data_token(
		&self,
		token: &Token,
	) -> Result<(Session, User), Error> {
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
