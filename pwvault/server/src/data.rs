use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Password {
	pub id: UniqueId,
	pub site: String,
	pub domain: String,
	pub username: String,
	/// needs to be encrypted from the user site
	pub password: String,
	pub created_on: DateTime,
}
