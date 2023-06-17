pub mod ffi;

pub mod stream;
pub mod server;
pub mod client;
pub mod fire;
pub mod progress_channel;

pub mod users;
pub mod config;

mod util;

use users::Sessions;

use std::{io, fmt};

pub use tokio::runtime;


pub struct Core {
	pub config: String,
	pub version: CoreVersion,
	pub on_terminate: server::OnTerminate,
	pub listener: stream::Listener,
	pub sessions: Sessions
}

impl Core {
	pub fn parse_config<T>(&self) -> Result<T, toml::de::Error>
	where T: serde::de::DeserializeOwned {
		toml::from_str(&self.config)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreVersion {
	pub major: u16,
	pub minor: u16
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub msg: String
}

impl Error {
	pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
		Self { kind, msg: msg.into() }
	}

	/// Panics if the c_error is Ok
	pub fn from_c(e: ffi::c_error) -> Self {
		Self {
			kind: ErrorKind::from_code(e.code),
			msg: unsafe { e.string.into_string() }
		}
	}

	pub fn into_c(self) -> ffi::c_error {
		ffi::c_error {
			code: self.kind.to_code(),
			string: ffi::c_string::from_string(self.msg)
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
	Broken,
	Closed,
	Refused,
	TooManyRequests,
	Other
}

impl ErrorKind {
	/// don't call with C_ERROR_OK
	pub fn from_code(code: u16) -> Self {
		assert!(code != 0);

		match code {
			ffi::C_ERROR_BROKEN => Self::Broken,
			ffi::C_ERROR_CLOSED => Self::Closed,
			ffi::C_ERROR_REFUSED => Self::Refused,
			ffi::C_ERROR_TOO_MANY_REQUESTS => Self::TooManyRequests,
			_ => Self::Other
		}
	}

	pub fn to_code(&self) -> u16 {
		match self {
			Self::Broken => ffi::C_ERROR_BROKEN,
			Self::Closed => ffi::C_ERROR_CLOSED,
			Self::Refused => ffi::C_ERROR_REFUSED,
			Self::TooManyRequests => ffi::C_ERROR_TOO_MANY_REQUESTS,
			Self::Other => ffi::C_ERROR_OTHER
		}
	}

	pub fn to_io(&self) -> io::ErrorKind {
		match self {
			Self::Broken => io::ErrorKind::BrokenPipe,
			Self::Closed => io::ErrorKind::ConnectionAborted,
			Self::Refused => io::ErrorKind::ConnectionRefused,
			// not the best response but meh.. (replace with resourceBusy once
			// stable)
			Self::TooManyRequests => io::ErrorKind::WouldBlock,
			Self::Other => io::ErrorKind::Other
		}
	}
}

/// ```
/// use core_lib::{init_fn, Core};
/// 
/// init_fn!(init, "some_app");
/// async fn init(core: Core) {
/// 	todo!()
/// }
/// ```
#[macro_export]
macro_rules! init_fn {
	($init:ident, $name:expr) => (
		$crate::init_fn!($init, $name, "", "");
	);
	($init:ident, $name:expr, $js_entry:expr) => (
		$crate::init_fn!($init, $name, $js_entry, "");
	);
	($init:ident, $name:expr, $js_entry:expr, $css_entry:expr) => (
		#[no_mangle]
		pub extern "C" fn c_init(
			core: *mut $crate::ffi::c_core,
			app: *mut $crate::ffi::c_app
		) {
			use $crate::ffi;

			let core = unsafe { &mut *core };
			let terminated = $crate::server::Terminated::new(
				core.terminated.take()
			);

			// init terminator
			let (terminator, terminate_rx) = $crate::client::Terminator::new();

			// init listener
			let (listener, c_listener) = $crate::stream::Listener::new();

			unsafe {
				app.write(ffi::c_app {
					name: ffi::c_str::from_str($name),
					js_entry: ffi::c_str::from_str($js_entry),
					css_entry: ffi::c_str::from_str($css_entry),
					terminator: terminator.into_c(),
					listener: c_listener.into_c()
				});
			}

			let config = unsafe { core.config.to_str().to_string() };
			let version = $crate::CoreVersion {
				major: core.version.major,
				minor: core.version.minor
			};
			let sessions = $crate::users::Sessions::new(core.sessions.take());

			std::thread::Builder::new()
				.name($name.into())
				.spawn(move || {
					let _ = std::panic::catch_unwind(
						std::panic::AssertUnwindSafe(move || {
							let core = $crate::Core {
								config, version,
								on_terminate: terminate_rx,
								listener, sessions
							};

							let rt = $crate::runtime::Runtime::new().unwrap();
							rt.block_on(async move {
								let _: () = $init(core).await;
							});
						})
					);

					// the runtime as stopped we should be able to call terminated
					// now
					terminated.terminated();
				}).unwrap();
		}
	)
}