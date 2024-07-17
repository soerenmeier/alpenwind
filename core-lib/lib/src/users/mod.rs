mod checked_user;
mod data;
#[cfg(not(feature = "i-am-core"))]
mod db;
#[cfg(feature = "i-am-core")]
pub mod db;
mod sessions;
mod timeout;
mod users;

use std::error::Error as StdError;

pub use checked_user::{
	CheckedUser, DataToken, NormalToken, RightsAny, RightsRoot,
};
pub use data::{Rights, Session, Token, User};
pub use sessions::Sessions;
pub use timeout::Timeout;
pub use users::Users;

use std::fmt;

use chuchi::{
	error::{ClientErrorKind, ErrorKind, ServerErrorKind},
	extractor::ExtractorError,
	header::RequestHeader,
};

pub fn get_token(header: &RequestHeader) -> Option<Token> {
	header.value("auth-token").and_then(|t| t.parse().ok())
}

pub fn get_token_from_cookie(header: &RequestHeader) -> Option<Token> {
	header
		.value("cookie")
		.and_then(|v| v.trim().strip_prefix("data-token="))
		.and_then(|t| t.trim().parse().ok())
}

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

impl ExtractorError for Error {
	fn error_kind(&self) -> ErrorKind {
		match self {
			Self::MissingAuthToken | Self::MissingDataToken => {
				ClientErrorKind::BadRequest.into()
			}
			Self::InvalidAuthToken
			| Self::InvalidDataToken
			| Self::InvalidUser => ClientErrorKind::Forbidden.into(),
			Self::Db(_) => ServerErrorKind::InternalServerError.into(),
		}
	}

	fn into_std(self) -> Box<dyn StdError + Send + Sync> {
		Box::new(self)
	}
}
