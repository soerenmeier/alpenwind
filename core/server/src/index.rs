use fire::fs::serve_memory_file;
use fire::header::{Method, RequestHeader};
use fire::routes::Route;
use fire::util::PinnedFuture;
use fire::{Data, Error, Request, Response, Result};

const FILE: &[u8] = include_bytes!("../../ui/dist/index.html");

pub struct Index;

impl Route for Index {
	fn check(&self, req: &RequestHeader) -> bool {
		req.method() == Method::GET
	}

	fn validate_data(&self, _data: &Data) {}

	fn call<'a>(
		&'a self,
		req: &'a mut Request,
		_data: &'a Data,
	) -> PinnedFuture<'a, Result<Response>> {
		PinnedFuture::new(async move {
			serve_memory_file("index.html", FILE, req, None)
				.map_err(Error::from_client_io)
		})
	}
}
