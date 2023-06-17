use super::{prog, MODULE_EXTENSION};
use crate::Users;
use crate::tempfile::TempFile;

use std::sync::Arc;
use std::mem::MaybeUninit;

use tokio::time::{self, Duration};
use tokio::runtime::Handle;

use core_lib::ffi;
use core_lib::stream::Connector;

use libloading::Library;


pub const RUNNNIG: usize = 0;
pub const TERMINATING: usize = 1;
pub const TERMINATED: usize = 2;



pub struct AppLib {
	pub connector: Connector,
	pub name: &'static str,
	pub js_entry: &'static str,
	pub css_entry: &'static str,
	pub terminated: prog::Receiver,
	pub terminator: Terminator
}

struct Lib {
	lib: Library,
	// used to cleanup the tmp file
	#[allow(dead_code)]
	file: TempFile
}

impl AppLib {
	pub fn new(path: &str, cfg: &str, users: &Users) -> Self {
		let file = TempFile::new(MODULE_EXTENSION).unwrap();
		std::fs::copy(path, &file).unwrap();

		let lib = Arc::new(Lib {
			lib: unsafe { Library::new(file.as_path()).unwrap() },
			file
		});
		let (term_tx, term_rx) = prog::channel(RUNNNIG);

		let terminated_ctx = Box::new(TerminatedCtx {
			lib: lib.clone(),
			notify: term_tx.clone(),
			handle: Handle::current()
		});

		let c_init = unsafe {
			lib.lib.get::<ffi::c_init_fn>(b"c_init")
				.unwrap()
		};

		// setup terminated
		extern "C" fn terminated_fn(ctx: *mut u8) {
			let ctx = unsafe { Box::from_raw(ctx as *mut TerminatedCtx) };
			ctx.notify.send(TERMINATED);

			ctx.handle.clone().spawn(async move {
				time::sleep(Duration::from_secs(2)).await;

				if let Ok(lib) = Arc::try_unwrap(ctx.lib) {
					if let Err(e) = lib.lib.close() {
						eprintln!("closing lib failed with {e:?}");
					}
				}
			});
		}
		let c_terminated = ffi::c_terminated {
			ctx: Box::into_raw(terminated_ctx) as *mut u8,
			terminated: terminated_fn
		};

		let mut core = ffi::c_core {
			config: ffi::c_str::from_str(cfg),
			version: ffi::c_core_version {
				major: 0,
				minor: 1
			},
			sessions: users.to_sessions_c(),
			terminated: c_terminated
		};

		let mut app = MaybeUninit::uninit();

		c_init(&mut core as *mut _, app.as_mut_ptr());

		let app = unsafe { app.assume_init() };

		Self {
			connector: Connector::new(app.listener),
			name: unsafe { app.name.to_static_str() },
			js_entry: unsafe { app.js_entry.to_static_str() },
			css_entry: unsafe { app.css_entry.to_static_str() },
			terminated: term_rx,
			terminator: Terminator {
				inner: app.terminator,
				notify: term_tx
			}
		}
	}
}

struct TerminatedCtx {
	lib: Arc<Lib>,
	notify: prog::Sender,
	handle: Handle
}

pub struct Terminator {
	inner: ffi::c_terminator,
	notify: prog::Sender
}

impl Terminator {
	pub fn terminate(self) {
		(self.inner.terminate)(self.inner.ctx);
		self.notify.send(TERMINATING);
	}
}

unsafe impl Send for Terminator {}
unsafe impl Sync for Terminator {}