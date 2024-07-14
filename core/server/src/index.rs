use chuchi::fs::serve_memory_file;
use chuchi::Chuchi;
use chuchi::{get, Error, Request, Response, Result};

const FILE: &[u8] = include_bytes!("../../ui/dist/index.html");

#[get("/")]
fn index(req: &mut Request) -> Result<Response> {
	serve_memory_file("index.html", FILE, req, None)
		.map_err(Error::from_client_io)
}

#[get("/{*rest}")]
fn index_rest(req: &mut Request) -> Result<Response> {
	serve_memory_file("index.html", FILE, req, None)
		.map_err(Error::from_client_io)
}

pub fn add_routes(server: &mut Chuchi) {
	server.add_route(index);
	server.add_route(index_rest);
}
