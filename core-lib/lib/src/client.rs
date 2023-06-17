use crate::{ffi, progress_channel as prog};
use crate::server::OnTerminate;

pub use tokio::runtime;

#[derive(Debug)]
pub struct Terminator {
	tx: prog::Sender
}

impl Terminator {
	pub fn new() -> (Self, OnTerminate) {
		let (tx, rx) = prog::channel(0);
		(Self { tx }, OnTerminate { rx })
	}

	pub fn into_c(self) -> ffi::c_terminator {
		let ctx = self.tx.into_raw() as *mut u8;

		/// only allowed to be called once
		extern "C" fn terminate(ctx: *mut u8) {
			let tx = unsafe { prog::Sender::from_raw(ctx as *const _) };
			tx.send(1);
		}

		ffi::c_terminator { ctx, terminate }
	}
}