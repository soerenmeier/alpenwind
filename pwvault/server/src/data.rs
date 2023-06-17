use postgres::time::DateTime;
use postgres::UniqueId;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Password {
	pub id: UniqueId,
	pub site: String,
	pub domain: String,
	pub username: String,
	/// needs to be encrypted from the user site
	pub password: String,
	pub created_on: DateTime
}