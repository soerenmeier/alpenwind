use crate::stream::Listener;

use std::future::Future;

use fire::FireBuilder;


pub async fn build() -> FireBuilder {
	fire::build("127.0.0.1:0").await.unwrap()
}

pub async fn ignite(
	builder: FireBuilder,
	listener: Listener,
	shutdown: impl Future<Output = ()>
) -> fire::Result<()> {
	hyper::Server::builder(listener)
		.serve(builder.into_make_fire_service())
		.with_graceful_shutdown(shutdown).await
		.map_err(fire::Error::from_server_error)
}


// pub async fn request<C>(
// 	connector: C,
// 	req: HyperRequest
// ) -> hyper::Result<HyperResponse>
// where C: Connect + Clone {
// 	let res = hyper::Client::builder()
// 		.build(connector)
// 		.request(req).await?;

// 	if res.status() == StatusCode::SWITCHING_PROTOCOLS {
// 		let upgraded = hyper::upgrade::on(res).await?;
// 		tokio::spawn(async move {
			
// 		});
// 	}

// }