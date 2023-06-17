mod api;
pub mod api_routes;
mod app_lib;
pub mod route;

use app_lib::{AppLib, Terminator};

use crate::Users;

use fire::Data;

use std::{io, mem};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{SystemTime, Instant};
use std::task::{Poll, Context};
use std::future::{self, Future};
use std::pin::Pin;
use std::convert::Infallible;
use std::borrow::Borrow;

use tokio::fs;
use tokio::time::{self, Duration};
use tokio::task::JoinHandle;

use hyper::service::Service;
use http::uri::{Uri, Scheme, Authority};

use core_lib::stream::{Connector, Stream};
use core_lib::progress_channel as prog;

use serde::{Serialize, Deserialize};

type HyperRequest = hyper::Request<hyper::Body>;
type HyperResponse = hyper::Response<hyper::Body>;

const MIN_RUNTIME: Duration = Duration::from_secs(4);

#[cfg(unix)]
const MODULE_EXTENSION: &str = "so";
#[cfg(windows)]
const MODULE_EXTENSION: &str = "dll";


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConf {
	// {dir}/{app}/{app}.so
	dir: Option<String>,
	#[serde(default)]
	files: Vec<String>
}

#[derive(Clone)]
pub struct Apps {
	inner: Arc<RwLock<AppsInner>>
}

impl Apps {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(RwLock::new(AppsInner::new()))
		}
	}

	fn exists(&self, name: &str) -> bool {
		let inner = self.inner.read().unwrap();
		inner.inner.contains_key(name)
	}

	pub fn get(&self, app: &str) -> Option<App> {
		let inner = self.inner.read().unwrap();
		inner.inner.get(app).map(Clone::clone)
	}

	pub fn to_api_apps(&self) -> Vec<api::App> {
		let inner = self.inner.read().unwrap();
		inner.inner.values().map(|a| {
			let inner = &a.inner;
			api::App {
				key: inner.name.to_string(),
				js_entry: Some(inner.js_entry.to_string())
					.filter(|j| !j.is_empty()),
				css_entry: Some(inner.css_entry.to_string())
					.filter(|c| !c.is_empty())
			}
		}).collect()
	}

	fn insert(&self, app: impl Into<App>) {
		let mut inner = self.inner.write().unwrap();
		let app = app.into();
		inner.inner.insert(app.inner.name, app);
	}

	fn remove(&self, name: &str) {
		let mut inner = self.inner.write().unwrap();
		inner.inner.remove(name);
	}
}

struct AppsInner {
	inner: HashMap<&'static str, App>
}

impl AppsInner {
	fn new() -> Self {
		Self {
			inner: HashMap::new()
		}
	}
}

#[derive(Clone)]
pub struct App {
	inner: Arc<AppInner>
}

impl App {
	pub async fn request(
		&self,
		mut req: HyperRequest
	) -> hyper::Result<HyperResponse> {
		// we need to set the scheme and authority since hyper requires it
		let uri = mem::take(req.uri_mut());
		let mut parts = uri.into_parts();
		parts.scheme = Some(Scheme::HTTP);
		parts.authority = Some(Authority::from_static("localhost"));
		*req.uri_mut() = Uri::from_parts(parts).unwrap();

		hyper::Client::builder()
			.build(self.clone())
			.request(req).await
	}
}

impl From<AppInner> for App {
	fn from(inner: AppInner) -> Self {
		Self {
			inner: Arc::new(inner)
		}
	}
}

impl Service<Uri> for App {
	type Response = Stream;
	type Error = Infallible;
	type Future = future::Ready<Result<Stream, Infallible>>;

	fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Infallible>> {
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, req: Uri) -> Self::Future {
		self.inner.connector.borrow().call(req)
	}
}

struct AppInner {
	name: &'static str,
	js_entry: &'static str,
	css_entry: &'static str,
	connector: Connector
}


async fn dir_files(path: &str) -> io::Result<Vec<String>> {
	let mut v = vec![];

	let mut read_dir = fs::read_dir(path).await?;
	while let Some(entry) = read_dir.next_entry().await? {
		let metadata = entry.metadata().await?;
		if !metadata.is_dir() {
			continue
		}

		let name = entry.file_name().into_string().unwrap();
		let mut path = entry.path();
		path.push(name);
		path.set_extension(MODULE_EXTENSION);

		v.push(path.into_os_string().into_string().unwrap());
	}

	Ok(v)
}

struct AppMetadata {
	last_modified: SystemTime,
	inserted: Instant,
	terminator: Option<Terminator>
}

pub(crate) fn bg_task(cfg: &AppsConf, data: Data) -> JoinHandle<()> {
	let cfg = cfg.clone();
	tokio::spawn(async move {
		let mut intv = time::interval(Duration::from_secs(10));
		intv.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
		let apps = data.get::<Apps>().unwrap();
		let users = data.get::<Users>().unwrap();
		let cfg_string = data.get::<crate::ConfigString>().unwrap();

		let mut raw_apps: HashMap<String, AppMetadata> = HashMap::new();

		let mut notifiers = Notifiers::new();

		loop {

			let mut files = if let Some(path) = &cfg.dir {
				dir_files(path).await.unwrap()
			} else {
				vec![]
			};

			files.extend_from_slice(&cfg.files);

			for file in files {
				let metadata = fs::metadata(&file).await.unwrap();
				let modified = metadata.modified().unwrap();

				if let Some(raw_app) = raw_apps.get_mut(&file) {
					if raw_app.last_modified == modified {
						continue
					}

					// the app already existed but was changed terminate
					if let Some(terminator) = raw_app.terminator.take() {
						terminator.terminate();
					}

					continue
				}

				// now create the AppLib
				let lib = AppLib::new(&file, &cfg_string.0, &users);

				eprintln!("enabling {:?} with file {file:?}", lib.name);

				raw_apps.insert(
					file.clone(),
					AppMetadata {
						last_modified: modified,
						inserted: Instant::now(),
						terminator: Some(lib.terminator)
					}
				);

				apps.insert(AppInner {
					name: lib.name,
					js_entry: lib.js_entry,
					css_entry: lib.css_entry,
					connector: lib.connector
				});

				notifiers.push(NotifiedApp {
					name: lib.name,
					file: file,
					notify: lib.terminated
				});
			}

			tokio::select! {
				_ = intv.tick() => {},
				idx = notifiers.notified(), if !notifiers.is_empty() => {
					let app = notifiers.get(idx);
					let state = app.notify.val();

					// Termination sent
					if state == app_lib::TERMINATING {
						eprintln!("app {:?} terminating", app.name);

						// remove it from the app list
						apps.remove(app.name);
					// Terminated
					} else if state >= app_lib::TERMINATED {
						eprintln!("app {:?} terminated", app.name);
						let app = notifiers.take(idx);

						apps.remove(app.name);

						let metadata = raw_apps.remove(&app.file).unwrap();
						if metadata.inserted.elapsed() < MIN_RUNTIME {
							// todo this blocks the entire app finding "process"
							time::sleep(Duration::from_secs(2)).await;
						}
					}
				}
			}
		}
	})
}

struct NotifiedApp {
	pub name: &'static str,
	pub file: String,
	pub notify: prog::Receiver
}

struct Notifiers {
	inner: Vec<NotifiedApp>
}

impl Notifiers {
	pub fn new() -> Self {
		Self {
			inner: vec![]
		}
	}

	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}

	pub fn get(&self, idx: usize) -> &NotifiedApp {
		&self.inner[idx]
	}

	/// terminated should be set to false when inserted
	pub fn push(&mut self, app: NotifiedApp) {
		self.inner.push(app);
	}

	pub fn take(&mut self, id: usize) -> NotifiedApp {
		self.inner.swap_remove(id)
	}

	/// the same as async fn notified(&mut self) -> usize
	pub fn notified(&mut self) -> Notified {
		Notified::new(self)
	}
}

struct Notified<'a> {
	/// this list and all it's values are pinned
	list: Vec<prog::Changed<'a>>
}

impl<'a> Notified<'a> {
	pub fn new(inner: &'a mut Notifiers) -> Self {
		Self {
			list: inner.inner.iter_mut().map(|n| n.notify.changed()).collect()
		}
	}
}

impl<'a> Future for Notified<'a> {
	type Output = usize;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<usize> {
		assert!(!self.list.is_empty());

		for (i, notified) in self.list.iter_mut().enumerate() {
			// notified never get's moved
			let notified = unsafe { Pin::new_unchecked(notified) };
			match notified.poll(cx) {
				Poll::Ready(_) => return Poll::Ready(i),
				Poll::Pending => {}
			}
		}

		Poll::Pending
	}
}

impl Unpin for Notified<'_> {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_notifiers() {
		let rt = tokio::runtime::Builder::new_current_thread()
			.enable_time()
			.build().unwrap();

		rt.block_on(async move {
			let (tx, rx) = prog::channel(0);

			let app = NotifiedApp {
				name: "hey",
				file: "hey".into(),
				notify: rx.clone()
			};

			let mut notifiers = Notifiers::new();
			notifiers.push(app);

			// notify
			tx.send(1);
			let idx = notifiers.notified().await;
			let a = notifiers.take(idx);
			notifiers.push(a);

			// notified fut
			let task = tokio::spawn(async move {
				time::sleep(Duration::from_millis(10)).await;
				tx.send(2);
			});

			let a = notifiers.notified().await;
			assert_eq!(a, 0);

			task.await.unwrap();
		});
	}
}