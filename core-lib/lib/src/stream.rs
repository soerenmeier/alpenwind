use crate::{ffi, util, Error, ErrorKind};

use std::{io, future};
use std::sync::Arc;
use std::pin::Pin;
use std::mem::MaybeUninit;
use std::task::{ready, Poll, Context};
use std::convert::Infallible;

use tokio::sync::{mpsc};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use bytes::Bytes;

use hyper::server::accept::Accept;
use hyper::service::Service;
use hyper::Uri;
use hyper::client::connect::{Connection, Connected};

use fire::service::{MakeFireService, FireService};

const CONCURRENT_STREAM_REQS: usize = 1024;
const CONCURRENT_WRITES_PER_STREAM: usize = 1024;

pub struct Listener {
	rx: mpsc::Receiver<Stream>
}

impl Listener {
	pub fn new() -> (Listener, CListener) {
		let (tx, rx) = mpsc::channel(CONCURRENT_STREAM_REQS);
		(Listener { rx }, CListener { tx })
	}

	pub async fn accept(&mut self) -> Option<Stream> {
		self.rx.recv().await
	}
}

impl Accept for Listener {
	type Conn = Stream;
	type Error = Error;

	fn poll_accept(
		mut self: Pin<&mut Self>,
		cx: &mut Context
	) -> Poll<Option<Result<Stream, Error>>> {
		self.rx.poll_recv(cx)
			.map(|o| o.map(Ok))
	}
}

impl<'a> Service<&'a Stream> for MakeFireService {
	type Response = FireService;
	type Error = Infallible;
	type Future = future::Ready<Result<FireService, Infallible>>;

	fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Infallible>> {
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, _stream: &'a Stream) -> Self::Future {
		future::ready(Ok(self.make(([127, 0, 0, 1], 0).into())))
	}
}

/// Used to create a c_listener
pub struct CListener {
	tx: mpsc::Sender<Stream>
}

impl CListener {
	pub fn into_c(self) -> ffi::c_listener {
		let ctx = Arc::into_raw(Arc::new(self)) as *mut u8;

		extern "C" fn accept(
			ctx: *const u8,
			reader: *mut ffi::c_writer,
			writer: ffi::c_writer
		) -> ffi::c_error {
			// we do a manual Arc::clone
			unsafe { Arc::increment_strong_count(ctx as *const CListener) };
			let ctx = unsafe { Arc::from_raw(ctx as *const CListener) };

			let (n_reader, c_reader) = Reader::new();
			unsafe { reader.write(c_reader.into_c()) };

			let writer = Writer::new(writer);

			let stream = Stream {
				reader: n_reader,
				writer
			};

			match util::mpsc_send(&ctx.tx, stream) {
				Ok(_) => ffi::c_error::ok(),
				Err(e) => e.into_c()
			}
		}

		/// only allowed to be called once
		extern "C" fn free(ctx: *mut u8) {
			drop(unsafe { Arc::from_raw(ctx as *mut CListener) });
		}

		ffi::c_listener { ctx, accept, free }
	}
}

struct Reader {
	buf: Option<Bytes>,
	rx: mpsc::Receiver<Bytes>
}

impl Reader {
	pub fn new() -> (Reader, TxReader) {
		let (tx, rx) = mpsc::channel(CONCURRENT_WRITES_PER_STREAM);

		(Reader { buf: None, rx }, TxReader { tx })
	}
}

impl AsyncRead for Reader {
	// if read bytes is 0 this means the stream was closed and should not be
	// called again
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf
	) -> Poll<io::Result<()>> {
		let mut bytes = if let Some(b) = self.buf.take() {
			b
		} else {
			ready!(self.rx.poll_recv(cx))
				.ok_or_else(|| io::Error::new(
					io::ErrorKind::BrokenPipe,
					"mpsc was channel closed"
				))?
		};

		let len = buf.remaining().min(bytes.len());

		let to_send = bytes.split_to(len);
		buf.put_slice(&to_send);

		if !bytes.is_empty() {
			self.buf = Some(bytes);
		}

		Poll::Ready(Ok(()))
	}
}

struct TxReader {
	tx: mpsc::Sender<Bytes>
}

impl TxReader {
	fn into_c(self) -> ffi::c_writer {
		let ctx = Box::into_raw(Box::new(self)) as *mut u8;

		extern "C" fn write(
			ctx: *mut u8,
			bytes: ffi::c_slice<u8>
		) -> ffi::c_error {
			// we have exclusive access
			let ctx = unsafe { &mut *(ctx as *mut TxReader) };

			let bytes = Bytes::copy_from_slice(unsafe { bytes.to_slice() });

			match util::mpsc_send(&ctx.tx, bytes) {
				Ok(_) => ffi::c_error::ok(),
				Err(e) => e.into_c()
			}
		}

		/// only allowed to be called once
		extern "C" fn free(ctx: *mut u8) {
			drop(unsafe { Box::from_raw(ctx as *mut TxReader) });
		}

		ffi::c_writer { ctx, write, free }
	}
}

struct Writer {
	is_closed: bool,
	inner: ffi::c_writer
}

impl Writer {
	pub fn new(inner: ffi::c_writer) -> Self {
		Self {
			is_closed: false,
			inner
		}
	}

	pub fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
		assert!(!bytes.is_empty());

		if self.is_closed {
			return Err(Error::new(ErrorKind::Closed, ""));
		}

		let r = (self.inner.write)(
			self.inner.ctx,
			ffi::c_slice::from_slice(bytes)
		);

		if r.is_ok() {
			Ok(())
		} else {
			Err(Error::from_c(r))
		}
	}

	pub fn close(&mut self) {
		if !self.is_closed {
			let e = (self.inner.write)(self.inner.ctx, ffi::c_slice::empty());
			e.free();
			self.is_closed = true;
		}
	}
}

/// this is safe since the ctx of the writer get's only accessed by &mut
unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

impl Drop for Writer {
	fn drop(&mut self) {
		self.close();
		(self.inner.free)(self.inner.ctx);
	}
}

impl AsyncWrite for Writer {
	fn poll_write(
		mut self: Pin<&mut Self>,
		_cx: &mut Context,
		buf: &[u8]
	) -> Poll<io::Result<usize>> {
		let r = self.write(buf)
			.map(|_| buf.len())
			.map_err(|e| io::Error::new(e.kind.to_io(), e.msg));

		Poll::Ready(r)
	}

	fn poll_flush(
		self: Pin<&mut Self>,
		_cx: &mut Context
	) -> Poll<io::Result<()>> {
		Poll::Ready(Ok(()))
	}

	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		_cx: &mut Context
	) -> Poll<io::Result<()>> {
		self.close();

		Poll::Ready(Ok(()))
	}
}


pub struct Stream {
	reader: Reader,
	writer: Writer
}

impl Stream {
	pub fn close_sender(&mut self) {
		self.writer.close();
	}
}

impl AsyncRead for Stream {
	// if read bytes is 0 this means the stream was closed and should not be
	// called again
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf
	) -> Poll<io::Result<()>> {
		Pin::new(&mut self.reader).poll_read(cx, buf)
	}
}

impl AsyncWrite for Stream {
	fn poll_write(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &[u8]
	) -> Poll<io::Result<usize>> {
		Pin::new(&mut self.writer).poll_write(cx, buf)
	}

	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context
	) -> Poll<io::Result<()>> {
		Pin::new(&mut self.writer).poll_flush(cx)
	}

	fn poll_shutdown(
		mut self: Pin<&mut Self>,
		cx: &mut Context
	) -> Poll<io::Result<()>> {
		Pin::new(&mut self.writer).poll_shutdown(cx)
	}
}

impl Connection for Stream {
	fn connected(&self) -> Connected {
		Connected::new()
	}
}

pub struct Connector {
	inner: ffi::c_listener
}

impl Connector {
	pub fn new(inner: ffi::c_listener) -> Self {
		Self { inner }
	}

	pub fn connect(&self) -> Stream {
		let (reader, c_reader) = Reader::new();

		let mut writer = MaybeUninit::uninit();

		(self.inner.accept)(
			self.inner.ctx,
			&mut writer as *mut _ as *mut _,
			c_reader.into_c()
		);

		let writer = Writer::new(unsafe { writer.assume_init() });

		Stream { reader, writer }
	}
}

/// this is safe since the ctx of the connector get's only accessed by &mut
unsafe impl Send for Connector {}
unsafe impl Sync for Connector {}

impl Drop for Connector {
	fn drop(&mut self) {
		(self.inner.free)(self.inner.ctx)
	}
}

impl Service<Uri> for &Connector {
	type Response = Stream;
	type Error = Infallible;
	type Future = future::Ready<Result<Stream, Infallible>>;

	fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Infallible>> {
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, _: Uri) -> Self::Future {
		future::ready(Ok(self.connect()))
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	use tokio::io::{AsyncReadExt, AsyncWriteExt};

	#[tokio::test]
	async fn simple_read_write() {
		let (mut listener, c_listener) = Listener::new();
		let connector = Connector::new(c_listener.into_c());

		let join_handle = std::thread::spawn(move || {
			let rt = tokio::runtime::Runtime::new().unwrap();
			rt.block_on(async move {
				let mut stream = listener.accept().await.unwrap();
				loop {
					let mut v = vec![];

					let l = stream.read_buf(&mut v).await.unwrap();
					if l == 0 {
						break
					}

					stream.write_all(&v).await.unwrap();
					v.clear();
				}
			});
		});

		let mut stream = connector.connect();
		stream.write_all(b"hey").await.unwrap();
		stream.close_sender();
		let mut v = vec![];
		stream.read_to_end(&mut v).await.unwrap();
		assert_eq!(v, b"hey");

		join_handle.join().unwrap();
	}
}