use crate::{ffi, progress_channel as prog};

pub use tokio::runtime;


#[derive(Debug, Clone)]
pub struct OnTerminate {
	pub(super) rx: prog::Receiver
}

impl OnTerminate {
	/// resolved the future when the terminate fn was called
	pub fn on_terminate(&mut self) -> prog::Changed {
		// make sure that we only listen until 1 get's received
		self.rx.set_received_val(0);
		self.rx.changed()
	}
}

/// Should be triggered when the server as cleaned up everything
/// the server as a library can now be closed and removed from memory.
#[doc(hidden)]
pub struct Terminated {
	inner: ffi::c_terminated
}

impl Terminated {
	pub fn new(inner: ffi::c_terminated) -> Self {
		Self { inner }
	}

	pub fn terminated(self) {
		(self.inner.terminated)(self.inner.ctx)
	}
}

unsafe impl Send for Terminated {}
unsafe impl Sync for Terminated {}