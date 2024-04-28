use super::{Rights, User};

use postgres::json::Json;
use postgres::table::TableOwned;
use postgres::{filter, FromRow, ToRow};
use postgres::{Database, Result, TableTempl, UniqueId};

/// should only be used by core itself
#[derive(Debug, TableTempl, ToRow, FromRow)]
pub struct UnsafeUser {
	#[index(primary)]
	pub id: UniqueId,
	#[index(unique)]
	pub username: String,
	pub name: String,
	// hashed
	pub password: String,
	pub rights: Json<Rights>,
}

impl From<UnsafeUser> for User {
	fn from(u: UnsafeUser) -> Self {
		Self {
			id: u.id,
			username: u.username,
			name: u.name,
			rights: u.rights.0,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Users {
	table: TableOwned<UnsafeUser>,
}

impl Users {
	pub async fn new(db: &Database) -> Self {
		Self {
			table: db.table_owned("users").create().await,
		}
	}

	pub async fn by_id(&self, id: &UniqueId) -> Result<Option<User>> {
		self.table
			.find_one(filter!(id))
			.await
			.map(|opt| opt.map(Into::into))
	}
}
