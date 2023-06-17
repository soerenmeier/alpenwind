use fire::{FireBuilder, Request, Response, Data};
use fire::routes::Catcher;
use fire::header::{StatusCode, RequestHeader, ResponseHeader};
use fire::util::PinnedFuture;
use fire::header::{
	Method, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
	ACCESS_CONTROL_ALLOW_HEADERS, X_XSS_PROTECTION,
	ACCESS_CONTROL_ALLOW_CREDENTIALS
};


struct CorsHeaders;

impl Catcher for CorsHeaders {
	fn check(&self, _req: &RequestHeader, _res: &ResponseHeader) -> bool {
		true
	}

	fn call<'a>(
		&'a self,
		req: &'a mut Request,
		res: &'a mut Response,
		_data: &'a Data
	) -> PinnedFuture<'a, fire::Result<()>> {
		let values = &mut res.header.values;

		// if we have a options request this means we need to
		// answer with access-control-allow-origin
		if req.header().method == Method::OPTIONS {
			res.header.status_code = StatusCode::NO_CONTENT;
			values.insert(ACCESS_CONTROL_ALLOW_METHODS, "POST, PUT");
		}

		let host = req.header().value("origin").unwrap_or("*").to_string();

		values.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
		values.insert(ACCESS_CONTROL_ALLOW_ORIGIN, host);
		values.insert(
			ACCESS_CONTROL_ALLOW_HEADERS,
			"content-type, auth-token, cookie, credentials"
		);
		values.insert(X_XSS_PROTECTION, "0");

		PinnedFuture::new(async move { Ok(()) })
	}
}

pub fn add_routes(server: &mut FireBuilder) {
	server.add_catcher(CorsHeaders);
}