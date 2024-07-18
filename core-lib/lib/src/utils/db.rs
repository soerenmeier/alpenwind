use core::fmt;

use chuchi::{
	error::{ErrorKind, ServerErrorKind},
	extractor::{Extractor, ExtractorError},
	extractor_extract, extractor_prepare, extractor_validate,
};
use chuchi_postgres::{
	connection::ConnectionOwned, database::DatabaseError, Connection, Database,
};

#[derive(Debug)]
pub struct DbError(pub DatabaseError);

impl fmt::Display for DbError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl std::error::Error for DbError {}

impl ExtractorError for DbError {
	fn error_kind(&self) -> ErrorKind {
		ErrorKind::Server(ServerErrorKind::InternalServerError)
	}

	fn into_std(self) -> Box<dyn std::error::Error + Send + Sync> {
		Box::new(self)
	}
}

#[derive(Debug)]
pub struct ConnOwned(pub ConnectionOwned);

impl<'a, R> Extractor<'a, R> for ConnOwned {
	type Error = DbError;
	type Prepared = Self;

	extractor_validate!(|validate| {
		assert!(
			validate.resources.exists::<Database>(),
			"Db resource not found"
		);
	});

	extractor_prepare!(|prepare| {
		let db = prepare.resources.get::<Database>().unwrap();
		db.get().await.map(ConnOwned).map_err(DbError)
	});

	extractor_extract!(|extract| { Ok(extract.prepared) });
}

impl ConnOwned {
	pub fn conn(&self) -> Connection {
		self.0.connection()
	}
}
