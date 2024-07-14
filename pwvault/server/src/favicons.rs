use crate::PwVaultConf;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use chuchi::resources::Resources;
use tokio::fs;
use tokio::sync::RwLock;

use chuchi::fs::{serve_file, serve_memory_file, Caching};
use chuchi::header::url::Authority;
use chuchi::header::{Method, StatusCode};
use chuchi::routes::{ParamsNames, PathParams, Route, RoutePath};
use chuchi::util::PinnedFuture;
use chuchi::{Chuchi, Error, Request, Response, Result};

const URI: &str = "/assets/pwvault/favicons/{*domain}";
const DEFAULT_FAVICON_BYTES: &[u8] =
	include_bytes!("../../ui/assets/favicon.png");

pub struct Favicons {
	inner: Arc<RwLock<Inner>>,
}

impl Favicons {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(RwLock::new(Inner {
				inner: HashMap::new(),
			})),
		}
	}
}

struct Inner {
	// if the Path is None, this means that we weren't able to resolve the
	// domain
	inner: HashMap<String, Option<PathBuf>>,
}

struct FaviconsRoute {
	caching: Caching,
}

impl Route for FaviconsRoute {
	fn validate_requirements(
		&self,
		_params: &ParamsNames,
		_resources: &Resources,
	) {
	}

	fn path(&self) -> RoutePath {
		RoutePath {
			method: Some(Method::GET),
			path: URI.into(),
		}
	}

	fn call<'a>(
		&'a self,
		req: &'a mut Request,
		params: &'a PathParams,
		resources: &'a Resources,
	) -> PinnedFuture<'a, Result<Response>> {
		PinnedFuture::new(async move {
			if let Some(path) =
				route(req, params.get("domain").unwrap(), resources).await
			{
				serve_file(path, req, Some(self.caching.clone()))
					.await
					.map_err(Error::from_client_io)
			} else {
				serve_memory_file(
					"favicon.png",
					DEFAULT_FAVICON_BYTES,
					req,
					Some(self.caching.clone()),
				)
				.map_err(Error::from_client_io)
			}
		})
	}
}

pub fn add_routes(fire: &mut Chuchi) {
	fire.add_route(FaviconsRoute {
		caching: Caching::default(),
	});
}

async fn route(
	_req: &mut Request,
	domain: &str,
	resources: &Resources,
) -> Option<PathBuf> {
	// check if the domain is a valid domain
	let authority = Authority::from_str(&domain).ok()?;

	if domain != authority.host() {
		return None;
	}

	let favicons = resources.get::<Favicons>().unwrap();
	let cfg = resources.get::<PwVaultConf>().unwrap();
	{
		// let's first readlock it
		// then check if the file exists
		let read = favicons.inner.read().await;

		match read.inner.get(domain) {
			Some(Some(p)) => return Some(p.clone()),
			Some(None) => return None,
			None => {
				let path = Path::new(&cfg.favicons_dir).join(domain);
				// check if the file exists
				let is_file = fs::metadata(&path)
					.await
					.map(|m| m.is_file())
					.unwrap_or(false);
				if is_file {
					return Some(path);
				}

				// let's try to load the domain
			}
		}
	}

	// let's writelock
	let mut write = favicons.inner.write().await;

	// check if this was resolved since we locked the read
	match write.inner.get(domain) {
		Some(Some(p)) => return Some(p.clone()),
		Some(None) => return None,
		None => {}
	};

	// now let's load the favicon
	let favicon = get_favicon(domain, cfg).await;

	write.inner.insert(domain.to_string(), favicon.clone());

	favicon
}

/// returns the path of a favicon
async fn get_favicon(domain: &str, cfg: &PwVaultConf) -> Option<PathBuf> {
	let resp = reqwest::get(format!(
		"https://www.google.com/s2/favicons?domain={domain}&sz=128"
	))
	.await
	.ok()?;

	if resp.status() != StatusCode::OK {
		return None;
	}

	let bytes = resp.bytes().await.ok()?;

	let path = Path::new(&cfg.favicons_dir).join(domain);

	fs::write(&path, bytes).await.ok()?;

	Some(path)
}
