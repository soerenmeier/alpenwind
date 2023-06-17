use crate::api::Error;

use serde::{Serialize, Deserialize};

use fire_api::{Request, Method};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
	pub key: String,
	pub js_entry: Option<String>,
	pub css_entry: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppsReq;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apps {
	pub apps: Vec<App>
}

impl Request for AppsReq {
	type Response = Apps;
	type Error = Error;

	const PATH: &'static str = "/api/apps/list";
	const METHOD: Method = Method::GET;
}