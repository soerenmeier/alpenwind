use super::{Session, User};
use crate::api::Error;

use serde::{Deserialize, Serialize};

use chuchi::api::{Method, Request};

// Login

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginReq {
	pub username: String,
	pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
	pub user: User,
	pub session: Session,
}

impl Request for LoginReq {
	type Response = Login;
	type Error = Error;

	const PATH: &'static str = "/api/users/login";
	const METHOD: Method = Method::POST;
}

// Login by token

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginByTokenReq;

impl Request for LoginByTokenReq {
	type Response = Login;
	type Error = Error;

	const PATH: &'static str = "/api/users/loginbytoken";
	const METHOD: Method = Method::POST;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

// Renew session

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewReq;

impl Request for RenewReq {
	type Response = Login;
	type Error = Error;

	const PATH: &'static str = "/api/users/renew";
	const METHOD: Method = Method::POST;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

// Logout

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutReq;

impl Request for LogoutReq {
	type Response = ();
	type Error = Error;

	const PATH: &'static str = "/api/users/logout";
	const METHOD: Method = Method::POST;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

// Save admin

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReq {
	pub name: String,
	pub password: Option<String>,
}

impl Request for SaveReq {
	type Response = User;
	type Error = Error;

	const PATH: &'static str = "/api/users/save";
	const METHOD: Method = Method::POST;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}
