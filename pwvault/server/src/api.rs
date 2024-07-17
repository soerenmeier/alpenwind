use crate::data::Password;
use crate::error::Error;

use serde::{Deserialize, Serialize};

use chuchi_postgres::UniqueId;

use chuchi::api::{Method, Request};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllReq {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct All {
	pub list: Vec<Password>,
}

impl Request for AllReq {
	type Response = All;
	type Error = Error;

	const PATH: &'static str = "/api/pwvault/passwords";
	const METHOD: Method = Method::GET;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditReq {
	pub id: Option<UniqueId>,
	pub site: String,
	pub domain: String,
	pub username: String,
	pub password: String,
}

impl Request for EditReq {
	type Response = Password;
	type Error = Error;

	const PATH: &'static str = "/api/pwvault/edit";
	const METHOD: Method = Method::POST;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReq;

impl Request for DeleteReq {
	type Response = ();
	type Error = Error;

	const PATH: &'static str = "/api/pwvault/{id}/delete";
	const METHOD: Method = Method::DELETE;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}
