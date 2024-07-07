use fire::service::FireService;

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;

use hyper::body::Incoming;

pub type HyperRequest = hyper::Request<Incoming>;

use crate::server::OnTerminate;
use crate::stream::Listener;

use fire::FireBuilder;

pub async fn build() -> FireBuilder {
	fire::build("127.0.0.1:0").await.unwrap()
}

pub async fn ignite(
	builder: FireBuilder,
	mut listener: Listener,
	on_terminate: OnTerminate,
) -> fire::Result<()> {
	let pit = builder.into_pit();

	loop {
		let Some(stream) = listener.accept().await else {
			break Ok(());
		};

		let io = TokioIo::new(stream);
		let service = FireService::new(pit.clone(), ([127, 0, 0, 1], 0).into());
		let mut on_terminate = on_terminate.clone();

		tokio::task::spawn(async move {
			let builder = Builder::new(TokioExecutor::new());
			let conn = builder.serve_connection_with_upgrades(io, service);
			tokio::pin!(conn);
			let on_terminate = on_terminate.on_terminate();
			tokio::pin!(on_terminate);
			let mut terminated = false;

			loop {
				// wait until either the connection or the server is terminated
				tokio::select! {
					_ = &mut on_terminate, if !terminated => {
						conn.as_mut().graceful_shutdown();
						terminated = true;
					}
					res = &mut conn => {
						if let Err(err) = res {
							tracing::error!("Connection error: {err}");
						}
						break;
					}
				}
			}
		});
	}
}

#[cfg(test)]
mod tests {
	use std::{pin::Pin, time::Duration};

	use bytes::Bytes;
	use fire::{
		body::BodyHttp,
		post,
		types::http::uri::{Authority, Scheme},
		Body, Request, Response,
	};
	use futures::TryStreamExt;
	use hyper::Uri;
	use hyper_util::client::legacy::{Client, Error as ClientError};
	use tokio::{
		io,
		sync::{broadcast, watch},
		time::{sleep, timeout},
	};
	use tokio_stream::{wrappers::BroadcastStream, StreamExt};
	use tracing_test::traced_test;

	use crate::{client::Terminator, stream::Connector};

	use super::*;

	// let's test reading a post request and respondig to it
	#[post("/api/echo")]
	async fn echo(req: &mut Request) -> Response {
		let body = req.take_body();
		Response::builder().body(body).build()
	}

	fn req_echo(body: impl Into<Body>) -> hyper::Request<Pin<Box<BodyHttp>>> {
		hyper::Request::builder()
			.method("POST")
			.uri("http://localhost:8080/api/echo")
			.body(Box::pin(body.into().into_http_body()))
			.unwrap()
	}

	#[tokio::test]
	#[traced_test]
	async fn test_echo() {
		let mut fire = build().await;
		fire.add_route(echo);

		let (list, c_list) = Listener::new();
		let conn = Connector::new(c_list.into_c()).into_shared();

		let (_, on_terminate) = Terminator::new();

		tokio::spawn(async move {
			ignite(fire, list, on_terminate).await.unwrap();
		});

		let req = req_echo("Hello, World!");

		// we need to set the scheme and authority since hyper requires it
		let resp = Client::builder(TokioExecutor::new())
			.build(conn)
			.request(req)
			.await
			.unwrap();
		let body = Body::from_hyper(resp.into_body());
		let body = body.into_string().await.unwrap();

		assert_eq!(body, "Hello, World!");
	}

	#[tokio::test]
	#[traced_test]
	async fn test_streaming_echo() {
		let mut fire = build().await;
		fire.add_route(echo);

		let (list, c_list) = Listener::new();
		let conn = Connector::new(c_list.into_c()).into_shared();

		let (_, on_terminate) = Terminator::new();

		tokio::spawn(async move {
			ignite(fire, list, on_terminate).await.unwrap();
		});

		let (tx, rx) = broadcast::channel::<Bytes>(4);
		let rx = BroadcastStream::new(rx)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e));

		tokio::spawn(async move {
			sleep(Duration::from_millis(50)).await;
			tx.send(Bytes::from_static(b"Hello, World!\n")).unwrap();

			sleep(Duration::from_millis(50)).await;
			tx.send(Bytes::from_static(b"Hello, Alpenwind\n")).unwrap();
		});

		let req = req_echo(Body::from_async_bytes_streamer(rx));

		// we need to set the scheme and authority since hyper requires it
		let resp = Client::builder(TokioExecutor::new())
			.build(conn)
			.request(req)
			.await
			.unwrap();
		let body = Body::from_hyper(resp.into_body());
		let body = body.into_string().await.unwrap();

		assert_eq!(body, "Hello, World!\nHello, Alpenwind\n");
	}

	// pub async fn request(
	// 	&self,
	// 	mut req: HyperRequest,
	// ) -> Result<HyperResponse, ClientError> {
	// 	// we need to set the scheme and authority since hyper requires it
	// 	let uri = mem::take(req.uri_mut());
	// 	let mut parts = uri.into_parts();
	// 	parts.scheme = Some(Scheme::HTTP);
	// 	parts.authority = Some(Authority::from_static("localhost"));
	// 	*req.uri_mut() = Uri::from_parts(parts).unwrap();

	// 	Client::builder(TokioExecutor::new())
	// 		.build(self.clone())
	// 		.request(req)
	// 		.await
	// }
}
