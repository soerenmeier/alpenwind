use std::mem::MaybeUninit;

use crate::ffi;

use super::{Session, Token};

pub struct Sessions {
	inner: ffi::c_sessions,
}

impl Sessions {
	pub fn new(inner: ffi::c_sessions) -> Self {
		Self { inner }
	}

	pub fn by_token(&self, token: &Token) -> Option<Session> {
		let mut sess = MaybeUninit::uninit();
		let token = ffi::c_token::from_token(token.clone());
		let some =
			(self.inner.by_token)(self.inner.ctx, token, sess.as_mut_ptr());

		if some {
			Some(Session::from_c(unsafe { sess.assume_init() }))
		} else {
			None
		}
	}

	pub fn by_data_token(&self, token: &Token) -> Option<Session> {
		let mut sess = MaybeUninit::uninit();
		let token = ffi::c_token::from_token(token.clone());
		let some = (self.inner.by_data_token)(
			self.inner.ctx,
			token,
			sess.as_mut_ptr(),
		);

		if some {
			Some(Session::from_c(unsafe { sess.assume_init() }))
		} else {
			None
		}
	}
}

impl Drop for Sessions {
	fn drop(&mut self) {
		(self.inner.free)(self.inner.ctx as *mut _);
	}
}

unsafe impl Send for Sessions {}
unsafe impl Sync for Sessions {}
