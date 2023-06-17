use super::Apps;

use std::mem;

use fire::{Data, Response, Error};
use fire::routes::{RawRoute, HyperRequest};
use fire::util::PinnedFuture;
use fire::header::{
	ContentType, ResponseHeader, HeaderValues, Method, StatusCode
};

use tracing::error;

const PREFIXES: &[&str] = &["/api/", "/assets/"];


pub struct AppsRoute {
	data: Data
}

impl AppsRoute {
	pub fn new(data: Data) -> Self {
		Self { data }
	}
}

impl RawRoute for AppsRoute {
	fn check(&self, req: &HyperRequest) -> bool {
		if !matches!(*req.method(), Method::GET | Method::POST | Method::PUT |
			Method::DELETE) {
			return false
		}

		let apps = self.data.get::<Apps>().unwrap();

		let path = req.uri().path();

		for prefix in PREFIXES {
			let Some(path) = path.strip_prefix(prefix) else {
				continue
			};

			let mut path = path.split('/');
			let app = path.next().unwrap_or("");

			return apps.exists(app);
		}

		false
	}

	fn call<'a>(
		&'a self,
		req: &'a mut HyperRequest,
		data: &'a Data
	) -> PinnedFuture<'a, Option<fire::Result<Response>>> {
		let path = req.uri().path();
		let path = path.strip_prefix('/').unwrap_or(path);
		let mut path = path.split('/');

		let _prefix = path.next().unwrap();
		let app = path.next().unwrap();

		let apps = data.get::<Apps>().unwrap();
		let app = apps.get(app).unwrap();
		let mut new_req = HyperRequest::default();
		*new_req.uri_mut() = req.uri().clone();
		*new_req.method_mut() = req.method().clone();
		*new_req.headers_mut() = req.headers().clone();
		mem::swap(new_req.body_mut(), req.body_mut());

		PinnedFuture::new(async move {Some(async move {
			let mut res = app.request(new_req).await
				.map_err(Error::from_server_error)?;

			if res.status() == StatusCode::SWITCHING_PROTOCOLS {
				// since the other side was ok with an upgrade
				// let's do it
				let req_upgrade = hyper::upgrade::on(req);
				let res_upgrade = hyper::upgrade::on(&mut res);
				tokio::spawn(async move {
					let mut req_upgraded = match req_upgrade.await {
						Ok(o) => o,
						Err(e) => {
							error!("req upgrade failed {e:?}");
							return;
						}
					};
					let mut res_upgraded = match res_upgrade.await {
						Ok(o) => o,
						Err(e) => {
							error!("req upgrade failed {e:?}");
							return;
						}
					};

					let r = tokio::io::copy_bidirectional(
						&mut req_upgraded,
						&mut res_upgraded
					).await;

					if let Err(e) = r {
						error!("upgraded proxy failed {e:?}");
					}
				});
			}

			let (parts, body) = res.into_parts();

			Ok(Response {
				header: ResponseHeader {
					status_code: parts.status,
					content_type: ContentType::None,
					values: HeaderValues::from_inner(parts.headers)
				},
				body: body.into()
			})
		}.await)})
	}
}