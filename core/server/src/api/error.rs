use std::fmt;

use serde::{Serialize, Deserialize};

use fire_api::error::{ApiError, Error as ErrorTrait, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	LoginIncorrect,
	MissingAuthToken,
	InvalidAuthToken,
	MissingDataToken,
	InvalidDataToken,
	Internal(String),
	Request(String)
}

impl ApiError for Error {
	fn internal<E: ErrorTrait>(e: E) -> Self {
		Self::Internal(e.to_string())
	}

	fn request<E: ErrorTrait>(e: E) -> Self {
		Self::Request(e.to_string())
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::LoginIncorrect |
			Self::MissingAuthToken |
			Self::InvalidAuthToken |
			Self::MissingDataToken |
			Self::InvalidDataToken => StatusCode::FORBIDDEN,
			Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
			Self::Request(_) => StatusCode::BAD_REQUEST
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {}

impl From<postgres::Error> for Error {
	fn from(e: postgres::Error) -> Self {
		Self::Internal(e.to_string())
	}
}