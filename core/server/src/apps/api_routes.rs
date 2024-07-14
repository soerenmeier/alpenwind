use super::api::{Apps, AppsReq};
use crate::api::Result;

use chuchi::Chuchi;

use chuchi::api;

#[api(AppsReq)]
async fn apps_route(apps: &super::Apps) -> Result<Apps> {
	Ok(Apps {
		apps: apps.to_api_apps(),
	})
}

pub fn add_routes(server: &mut Chuchi) {
	server.add_route(apps_route);
}
