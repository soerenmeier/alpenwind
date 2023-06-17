use super::{User, Rights};

use postgres::{Result, Table, Database, UniqueId, TableTempl};
use postgres::{whr, impl_json_col_type};


/// should only be used by core itself
#[derive(Debug, TableTempl)]
pub struct UnsafeUser {
	#[index(primary)]
	pub id: UniqueId,
	#[index(unique)]
	pub username: String,
	pub name: String,
	// hashed
	pub password: String,
	pub rights: Rights
}

impl_json_col_type!(Rights);

impl From<UnsafeUser> for User {
	fn from(u: UnsafeUser) -> Self {
		let UnsafeUser { id, username, name, rights, .. } = u;
		Self { id, username, name, rights }
	}
}

#[derive(Debug, Clone)]
pub struct Users {
	table: Table<UnsafeUser>
}

impl Users {
	pub async fn new(db: &Database) -> Self {
		Self {
			table: db.table("users").create().await
		}
	}

	pub async fn by_id(&self, id: &UniqueId) -> Result<Option<User>> {
		self.table.find_one(whr!(id)).await
			.map(|opt| opt.map(Into::into))
	}
}