use super::api::{AppsReq, Apps};
use crate::api::Result;

use fire::FireBuilder;

use fire_api::api;


#[api(AppsReq)]
async fn apps_route(apps: &super::Apps) -> Result<Apps> {
	Ok(Apps {
		apps: apps.to_api_apps()
	})
}

pub fn add_routes(server: &mut FireBuilder) {
	server.add_route(apps_route);
}