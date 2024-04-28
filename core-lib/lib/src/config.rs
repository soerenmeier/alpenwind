use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConf {
	pub host: String,
	pub name: String,
	pub user: String,
	pub password: String,
}
