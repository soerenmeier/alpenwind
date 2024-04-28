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
