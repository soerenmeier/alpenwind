//! A progress channel is a channel which waits on progress
//! the progress here is a usize which is growing

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use std::future::Future;
use std::pin::Pin;

use tokio::sync::{Notify, futures};

use pin_project_lite::pin_project;


pub fn channel(val: usize) -> (Sender, Receiver) {
	let inner = Arc::new(Inner {
		notify: Notify::new(),
		val: AtomicUsize::new(val)
	});

	(Sender { inner: inner.clone() }, Receiver { inner, received: val })
}

#[derive(Debug)]
struct Inner {
	notify: Notify,
	val: AtomicUsize
}

impl Inner {
	 pub fn val(&self) -> usize {
	 	self.val.load(Ordering::Relaxed)
	 }
}

#[derive(Debug, Clone)]
pub struct Sender {
	inner: Arc<Inner>
}

impl Sender {
	pub fn val(&self) -> usize {
		self.inner.val()
	}

	pub fn into_raw(self) -> *const u8 {
		Arc::into_raw(self.inner) as *const u8
	}

	pub unsafe fn from_raw(ptr: *const u8) -> Self {
		Self {
			inner: Arc::from_raw(ptr as *const _)
		}
	}

	/// sends the value if it is higher than the already stored one
	/// 
	/// Returns true if sent
	pub fn send(&self, val: usize) -> bool {
		let prev = self.inner.val.fetch_max(val, Ordering::Relaxed);
		let sent = prev < val;
		if sent {
			self.inner.notify.notify_waiters();
		}

		sent
	}
}

#[derive(Debug, Clone)]
pub struct Receiver {
	inner: Arc<Inner>,
	received: usize
}

impl Receiver {
	pub fn val(&self) -> usize {
		self.inner.val()
	}

	pub fn set_received_val(&mut self, val: usize) {
		self.received = val;
	}

	pub fn received_val(&self) -> usize {
		self.received
	}

	/// When the future gets resolved this means that the val as increased from
	/// the received val and the received val was updated.
	pub fn changed(&mut self) -> Changed {
		Changed {
			notified: self.inner.notify.notified(),
			inner: &self.inner,
			was_ready: false,
			received: &mut self.received
		}
	}

	// pub fn notified(&self) -> futures::Notified {
	// 	self.inner.notify.notified()
	// }
}

pin_project! {
	#[derive(Debug)]
	pub struct Changed<'a> {
		inner: &'a Inner,
		received: &'a mut usize,
		was_ready: bool,
		#[pin]
		notified: futures::Notified<'a>
	}
}

impl Future for Changed<'_> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
		let mut me = self.project();

		loop {
			// check if the val is already bigger
			let val = me.inner.val();
			if val > **me.received {
				**me.received = val;
				return Poll::Ready(())
			}

			if *me.was_ready {
				me.notified.set(me.inner.notify.notified());
				*me.was_ready = false;
			}

			ready!(me.notified.as_mut().poll(cx));
			*me.was_ready = true;
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use tokio::time;


	#[test]
	fn progress_channel() {
		let rt = tokio::runtime::Builder::new_current_thread()
			.enable_time()
			.build().unwrap();

		rt.block_on(progress_channel_test());
	}

	async fn progress_channel_test() {
		let (tx, mut rx) = channel(0);

		// wait after the fact
		tx.send(1);

		rx.changed().await;
		assert_eq!(rx.val(), 1);
		assert_eq!(rx.received_val(), 1);

		let changed = rx.changed();

		tx.send(2);

		changed.await;
		assert_eq!(rx.val(), 2);
		assert_eq!(rx.received_val(), 2);

		let task = tokio::spawn(async move {
			rx.changed().await;

			assert_eq!(rx.val(), 3);
			assert_eq!(rx.received_val(), 3);
		});

		time::sleep(time::Duration::from_millis(50)).await;
		tx.send(2);
		time::sleep(time::Duration::from_millis(50)).await;
		tx.send(3);

		task.await.unwrap();
	}
}