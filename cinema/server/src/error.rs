use std::fmt;

use serde::{Deserialize, Serialize};

use chuchi::api::error::{self, Error as ApiError, StatusCode};

use core_lib::users;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	LoginIncorrect,
	MissingAuthToken,
	InvalidAuthToken,
	MissingDataToken,
	InvalidDataToken,
	InvalidUser,
	Internal(String),
	Request(String),
}

impl error::ApiError for Error {
	fn from_error(e: ApiError) -> Self {
		match e {
			ApiError::HeadersMissing(_) | ApiError::Deserialize(_) => {
				Self::Request(e.to_string())
			}
			e => Self::Internal(e.to_string()),
		}
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::LoginIncorrect
			| Self::MissingAuthToken
			| Self::InvalidAuthToken
			| Self::MissingDataToken
			| Self::InvalidDataToken
			| Self::InvalidUser => StatusCode::FORBIDDEN,
			Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
			Self::Request(_) => StatusCode::BAD_REQUEST,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {}

impl From<chuchi_postgres::Error> for Error {
	fn from(e: chuchi_postgres::Error) -> Self {
		Self::Internal(e.to_string())
	}
}

impl From<users::Error> for Error {
	fn from(e: users::Error) -> Self {
		use users::Error::*;

		match e {
			MissingAuthToken => Self::MissingAuthToken,
			MissingDataToken => Self::MissingDataToken,
			InvalidAuthToken => Self::InvalidAuthToken,
			InvalidDataToken => Self::InvalidDataToken,
			InvalidUser => Self::InvalidUser,
			Db(e) => Self::Internal(e.to_string()),
		}
	}
}
