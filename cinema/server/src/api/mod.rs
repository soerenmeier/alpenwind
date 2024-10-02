pub mod data;

use crate::error::Error;
use data::Entry;

use core_lib::users::Token;

use serde::{Deserialize, Serialize};

use chuchi_postgres::UniqueId;

use chuchi::api::stream::{Stream, StreamKind};
use chuchi::api::{Method, Request};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntriesReq {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entries {
	pub list: Vec<Entry>,
}

impl Request for EntriesReq {
	type Response = Entries;
	type Error = Error;

	const PATH: &'static str = "/api/cinema/entries";
	const METHOD: Method = Method::GET;
	const HEADERS: &'static [&'static str] = &["auth-token"];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressReq {
	pub token: Token,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id")]
pub enum ProgressId {
	Movie(UniqueId),
	Episode(UniqueId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMsg {
	pub id: ProgressId,
	pub percent: f32,
}

// stream is on /api/cinema/stream
impl Stream for ProgressReq {
	type Message = ProgressMsg;
	type Error = Error;

	const KIND: StreamKind = StreamKind::Sender;
	const ACTION: &'static str = "progress";
}
