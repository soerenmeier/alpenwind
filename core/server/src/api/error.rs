use std::fmt;

use serde::{Deserialize, Serialize};

use fire_api::error::{self, ApiError, StatusCode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	LoginIncorrect,
	MissingAuthToken,
	InvalidAuthToken,
	MissingDataToken,
	InvalidDataToken,
	Internal(String),
	Request(String),
}

impl ApiError for Error {
	fn from_error(e: error::Error) -> Self {
		use error::Error::*;

		match e {
			HeadersMissing(_) | Deserialize(_) => Self::Request(e.to_string()),
			ExtractionError(e) => {
				// we should check if the type is Error
				e.downcast()
					.map(|e| *e)
					.unwrap_or_else(|e| Self::Internal(e.to_string()))
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
			| Self::InvalidDataToken => StatusCode::FORBIDDEN,
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

impl From<postgres::Error> for Error {
	fn from(e: postgres::Error) -> Self {
		Self::Internal(e.to_string())
	}
}
