pub use super::timeout::Timeout;
use crate::ffi;

use postgres::UniqueId;
use postgres::time::DateTime;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	pub id: UniqueId,
	pub username: String,
	pub name: String,
	pub rights: Rights
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rights {
	pub root: bool
}

pub type Token = crypto::token::Token<32>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
	pub token: Token,
	pub data_token: Token,
	pub timeout: Timeout,
	pub created_on: DateTime,
	pub user_id: UniqueId
}

impl Session {
	#[cfg(feature = "i-am-core")]
	pub fn new(timeout: Timeout, user_id: UniqueId) -> Self {
		Self {
			token: Token::new(),
			data_token: Token::new(),
			timeout,
			created_on: DateTime::now(),
			user_id
		}
	}

	pub fn did_timeout(&self) -> bool {
		self.timeout.has_elapsed()
	}

	pub fn from_c(inner: ffi::c_session) -> Self {
		Self {
			token: inner.token.into_token(),
			data_token: inner.data_token.into_token(),
			timeout: Timeout::from_c(inner.timeout),
			created_on: inner.created_on.to_datetime(),
			user_id: inner.user_id.to_uid()
		}
	}

	pub fn into_c(self) -> ffi::c_session {
		ffi::c_session {
			token: ffi::c_token::from_token(self.token),
			data_token: ffi::c_token::from_token(self.data_token),
			timeout: self.timeout.into_c(),
			created_on: ffi::c_datetime::from_datetime(self.created_on),
			user_id: ffi::c_uid::from_uid(self.user_id)
		}
	}
}