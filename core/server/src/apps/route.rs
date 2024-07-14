use super::Apps;

use std::mem;
use std::net::SocketAddr;

use chuchi::header::{ContentType, HeaderValues, ResponseHeader, StatusCode};
use chuchi::resources::Resources;
use chuchi::routes::{HyperRequest, PathParams, RawRoute, RoutePath};
use chuchi::util::PinnedFuture;
use chuchi::{Error, Response};

use hyper_util::rt::TokioIo;
use tracing::error;

const PREFIXES: &[&str] = &["/api/", "/assets/"];

pub struct AppsRoute;

impl RawRoute for AppsRoute {
	fn path(&self) -> RoutePath {
		RoutePath {
			method: None,
			path: "/{*p}".into(),
		}
	}

	fn call<'a>(
		&'a self,
		req: &'a mut HyperRequest,
		_address: SocketAddr,
		_params: &'a PathParams,
		resources: &'a Resources,
	) -> PinnedFuture<'a, Option<chuchi::Result<Response>>> {
		let path = req.uri().path();

		// make sure we wan't to redirect the request
		if !PREFIXES.iter().any(|prefix| path.starts_with(prefix)) {
			return PinnedFuture::new(async { None });
		}

		let path = path.strip_prefix('/').unwrap_or(path);
		let mut path = path.split('/');

		let _prefix = path.next().unwrap();
		let app = path.next().unwrap();

		let apps = resources.get::<Apps>().unwrap();
		let Some(app) = apps.get(app) else {
			return PinnedFuture::new(async { None });
		};
		let mut new_req = hyper::Request::builder()
			.method(req.method().clone())
			.uri(req.uri().clone())
			.version(req.version())
			.body(req.body_mut().take())
			.unwrap();
		*new_req.headers_mut() = req.headers().clone();

		PinnedFuture::new(async move {
			let fut = async move {
				let mut res = app
					.request(new_req)
					.await
					.map_err(Error::from_server_error)?;

				if res.status() == StatusCode::SWITCHING_PROTOCOLS {
					// since the other side was ok with an upgrade
					// let's do it
					let req_upgrade = hyper::upgrade::on(req);
					let res_upgrade = hyper::upgrade::on(&mut res);
					tokio::spawn(async move {
						let mut req_upgraded = match req_upgrade.await {
							Ok(o) => TokioIo::new(o),
							Err(e) => {
								error!("req upgrade failed {e:?}");
								return;
							}
						};
						let mut res_upgraded = match res_upgrade.await {
							Ok(o) => TokioIo::new(o),
							Err(e) => {
								error!("req upgrade failed {e:?}");
								return;
							}
						};

						let r = tokio::io::copy_bidirectional(
							&mut req_upgraded,
							&mut res_upgraded,
						)
						.await;

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
						values: HeaderValues::from_inner(parts.headers),
					},
					body: body.into(),
				})
			}
			.await;
			Some(fut)
		})
	}
}
