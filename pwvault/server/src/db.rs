use super::data;

use chuchi::Resource;
use chuchi_postgres::table::TableOwned;
use chuchi_postgres::time::DateTime;
use chuchi_postgres::{
	filter, whr, Database, FromRow, Result, TableTempl, ToRow, UniqueId,
};

#[derive(Debug, TableTempl, FromRow, ToRow)]
pub struct Password {
	#[index(primary)]
	pub id: UniqueId,
	#[index(index)]
	pub user_id: UniqueId,
	pub site: String,
	pub domain: String,
	pub username: String,
	/// needs to be encrypted from the user site
	pub password: String,
	pub created_on: DateTime,
}

impl From<Password> for data::Password {
	fn from(p: Password) -> Self {
		Self {
			id: p.id,
			site: p.site,
			domain: p.domain,
			username: p.username,
			password: p.password,
			created_on: p.created_on,
		}
	}
}

#[derive(Resource)]
pub struct Passwords {
	table: TableOwned<Password>,
}

impl Passwords {
	pub async fn new(db: &Database) -> Self {
		Self {
			table: db.table_owned("pwvault").create().await,
		}
	}

	pub async fn all_by_user(
		&self,
		user_id: &UniqueId,
	) -> Result<Vec<data::Password>> {
		let entries = self.table.find_many(filter!(user_id)).await?;

		Ok(entries.into_iter().map(Into::into).collect())
	}

	pub async fn insert(&self, password: &Password) -> Result<()> {
		self.table.insert_one(password).await
	}

	pub async fn update(&self, password: &Password) -> Result<()> {
		let id = &password.id;
		let user_id = &password.user_id;
		self.table.update_full(password, whr!(id AND user_id)).await
	}

	pub async fn delete(
		&self,
		id: &UniqueId,
		user_id: &UniqueId,
	) -> Result<()> {
		self.table.delete(whr!(id AND user_id)).await
	}
}
