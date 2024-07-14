//! Terminology
//! Core is the client, and App is the Server.
//!
//! The names are in the perspective of the Server.

#![allow(non_camel_case_types)]

use std::mem::ManuallyDrop;
use std::time::{Duration, SystemTime};
use std::{mem, ptr, slice};

use crypto::token::Token;

use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

#[repr(C)]
pub struct c_str {
	/// Never allowed to be null
	pub ptr: *const u8,
	pub len: usize,
}

impl c_str {
	pub unsafe fn to_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(slice::from_raw_parts(
				self.ptr, self.len,
			))
		}
	}

	/// only safe if this was created with from_static_str
	pub unsafe fn to_static_str(&self) -> &'static str {
		unsafe {
			std::str::from_utf8_unchecked(slice::from_raw_parts(
				self.ptr, self.len,
			))
		}
	}

	pub fn from_str(s: &str) -> Self {
		Self {
			ptr: s.as_bytes().as_ptr(),
			len: s.as_bytes().len(),
		}
	}

	pub fn from_static_str(s: &'static str) -> Self {
		Self::from_str(s)
	}
}

/// You need to call into_string
/// else you will leak memory
#[repr(C)]
pub struct c_string {
	pub ptr: *mut u8,
	pub len: usize,
	pub cap: usize,
	pub free: extern "C" fn(ptr: *mut u8, len: usize, cap: usize),
}

impl c_string {
	pub fn empty() -> Self {
		Self::from_string(String::new())
	}

	pub fn from_string(s: String) -> Self {
		extern "C" fn free(ptr: *mut u8, len: usize, cap: usize) {
			unsafe { drop(String::from_raw_parts(ptr, len, cap)) }
		}

		// let (ptr, len, cap) = s.into_raw_parts();
		let s = ManuallyDrop::new(s);
		Self {
			ptr: s.as_ptr() as *mut _,
			len: s.len(),
			cap: s.capacity(),
			free,
		}
	}

	pub unsafe fn into_string(self) -> String {
		let s = {
			unsafe {
				std::str::from_utf8_unchecked(slice::from_raw_parts(
					self.ptr, self.len,
				))
			}
			.to_string()
		};

		(self.free)(self.ptr, self.len, self.cap);

		s
	}

	pub unsafe fn free(self) {
		(self.free)(self.ptr, self.len, self.cap);
	}
}

#[repr(C)]
pub struct c_slice<T> {
	/// never allowed to be null
	///
	/// Use NonNull::dangling() when using len == 0
	pub ptr: *const T,
	pub len: usize,
}

impl<T> c_slice<T> {
	/// Safety make sure the slice does not outlive the allowed lifetime
	pub unsafe fn to_slice(&self) -> &[T] {
		slice::from_raw_parts(self.ptr, self.len)
	}

	pub fn from_slice(s: &[T]) -> Self {
		Self {
			ptr: s.as_ptr(),
			len: s.len(),
		}
	}

	pub fn empty() -> Self {
		Self {
			ptr: ptr::NonNull::dangling().as_ptr(),
			len: 0,
		}
	}
}

#[repr(C)]
pub struct c_core_version {
	pub major: u16,
	pub minor: u16,
}

#[repr(C)]
pub struct c_terminator {
	pub ctx: *mut u8,
	/// get's only called once
	pub terminate: extern "C" fn(ctx: *mut u8),
}

impl Default for c_terminator {
	fn default() -> Self {
		extern "C" fn terminate(_: *mut u8) {}

		Self {
			ctx: ptr::null_mut(),
			terminate,
		}
	}
}

#[repr(C)]
pub struct c_terminated {
	pub ctx: *mut u8,
	/// get's only called once
	pub terminated: extern "C" fn(ctx: *mut u8),
}

impl c_terminated {
	pub fn take(&mut self) -> Self {
		mem::take(self)
	}
}

impl Default for c_terminated {
	fn default() -> Self {
		extern "C" fn terminated(_: *mut u8) {}

		Self {
			ctx: ptr::null_mut(),
			terminated,
		}
	}
}

/// needs to call free else you will leak memory
#[repr(C)]
pub struct c_error {
	pub code: u16,
	pub string: c_string,
}

impl c_error {
	pub fn ok() -> Self {
		Self {
			code: C_ERROR_OK,
			string: c_string::empty(),
		}
	}

	pub fn is_ok(&self) -> bool {
		self.code == C_ERROR_OK
	}

	pub fn new(code: u16, string: String) -> Self {
		Self {
			code,
			string: c_string::from_string(string),
		}
	}

	pub fn free(self) {
		unsafe { self.string.free() };
	}
}

pub const C_ERROR_OK: u16 = 0;
pub const C_ERROR_BROKEN: u16 = 10;
pub const C_ERROR_CLOSED: u16 = 12;
pub const C_ERROR_REFUSED: u16 = 14;
pub const C_ERROR_TOO_MANY_REQUESTS: u16 = 16;
pub const C_ERROR_OTHER: u16 = u16::MAX;

#[repr(C)]
pub struct c_writer {
	pub ctx: *mut u8,
	/// You get a &mut ctx (so nobody else has access while in the read fn)
	pub write: extern "C" fn(ctx: *mut u8, bytes: c_slice<u8>) -> c_error,
	pub free: extern "C" fn(ctx: *mut u8),
}

#[repr(C)]
pub struct c_listener {
	pub ctx: *mut u8,
	/// The listener accept fn get's called once a new connection should be
	/// accepted, the server must set the first writer so that it can be
	/// called by the client to write to the server.
	///
	/// This fn may be called from different threads and at the same time.
	pub accept:
		extern "C" fn(ctx: *const u8, *mut c_writer, c_writer) -> c_error,
	/// Get's called by the client once he does not intend to call accept again
	pub free: extern "C" fn(ctx: *mut u8),
}

impl Default for c_listener {
	fn default() -> Self {
		extern "C" fn accept(
			_: *const u8,
			_: *mut c_writer,
			_: c_writer,
		) -> c_error {
			c_error::ok()
		}
		extern "C" fn free(_: *mut u8) {}

		Self {
			ctx: ptr::null_mut(),
			accept,
			free,
		}
	}
}

#[repr(C)]
pub struct c_token {
	pub bytes: [u8; 32],
}

impl c_token {
	pub fn from_token(t: Token<32>) -> Self {
		Self {
			bytes: t.to_bytes(),
		}
	}

	pub fn into_token(self) -> Token<32> {
		self.bytes.into()
	}
}

#[repr(C)]
pub struct c_systemtime {
	pub secs: u64,
	pub nanos: u32,
}

impl c_systemtime {
	pub fn from_systemtime(time: SystemTime) -> Self {
		let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
		Self {
			secs: dur.as_secs(),
			nanos: dur.subsec_nanos(),
		}
	}

	pub fn to_systemtime(&self) -> SystemTime {
		SystemTime::UNIX_EPOCH + Duration::new(self.secs, self.nanos)
	}
}

#[repr(C)]
pub struct c_datetime {
	pub secs: i64,
	pub nanos: u32,
}

impl c_datetime {
	pub fn from_datetime(datetime: DateTime) -> Self {
		let naive = datetime.into_inner();

		Self {
			secs: naive.timestamp(),
			nanos: naive.timestamp_subsec_nanos(),
		}
	}

	pub fn to_datetime(&self) -> DateTime {
		DateTime::new(self.secs, self.nanos)
	}
}

#[repr(C)]
pub struct c_uid {
	bytes: [u8; 10],
}

impl c_uid {
	pub fn from_uid(uid: UniqueId) -> Self {
		Self {
			bytes: uid.into_bytes(),
		}
	}

	pub fn to_uid(&self) -> UniqueId {
		UniqueId::from_raw(self.bytes)
	}
}

#[repr(C)]
pub struct c_session {
	pub token: c_token,
	pub data_token: c_token,
	pub timeout: c_systemtime,
	pub created_on: c_datetime,
	pub user_id: c_uid,
}

#[repr(C)]
pub struct c_sessions {
	pub ctx: *const u8,
	pub by_token: extern "C" fn(*const u8, c_token, *mut c_session) -> bool,
	pub by_data_token:
		extern "C" fn(*const u8, c_token, *mut c_session) -> bool,
	pub free: extern "C" fn(*const u8),
}

impl c_sessions {
	pub fn take(&mut self) -> Self {
		mem::take(self)
	}
}

impl Default for c_sessions {
	fn default() -> Self {
		extern "C" fn by_token(
			_ctx: *const u8,
			_token: c_token,
			_session: *mut c_session,
		) -> bool {
			false
		}
		extern "C" fn by_data_token(
			_ctx: *const u8,
			_token: c_token,
			_session: *mut c_session,
		) -> bool {
			false
		}
		extern "C" fn free(_ctx: *const u8) {}

		Self {
			ctx: ptr::null(),
			by_token,
			by_data_token,
			free,
		}
	}
}

/// The server receives a pointer to this struct in the init call
/// Don't hold on to core beyond the init call
#[repr(C)]
pub struct c_core {
	/// config is the original toml str (this needs to be parsed for your
	/// special configuration)
	pub config: c_str,
	pub version: c_core_version,
	pub sessions: c_sessions,
	/// Gets provided by the client and should be called when it is safe
	/// to destroy all references to the server.
	pub terminated: c_terminated,
}

/// All this properties should be set by the app (the server)
///
/// when you receive it from c_init it is not initialized (you need to write
/// the struct to the ptr)
#[repr(C)]
pub struct c_app {
	/// You need to set the name of the app
	///
	/// This needs to be a static str
	pub name: c_str,
	/// You need to set the js entry of the app (might be empty)
	///
	/// This needs to be a static str
	pub js_entry: c_str,
	/// You need to set the css entry of the app (might be empty)
	///
	/// This needs to be a static str
	pub css_entry: c_str,
	/// The terminator gets called by client once he want's the server to close
	pub terminator: c_terminator,
	/// The listener accept fn get's called once a new connection should be
	/// accepted
	pub listener: c_listener,
}

#[allow(non_camel_case_types)]
pub type c_init_fn = extern "C" fn(*mut c_core, *mut c_app);
