use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entry {
	Movie(Movie),
	Series(Series),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
	pub id: UniqueId,
	pub name: String,
	pub year: u32,
	pub updated_on: DateTime,
	pub progress: Option<Progress>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
	pub percent: f32,
	// position in in seconds
	pub position: f32,
	pub updated_on: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
	pub id: UniqueId,
	pub name: String,
	pub seasons: Vec<Season>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
	pub name: Option<String>,
	pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
	pub name: String,
	pub updated_on: DateTime,
	pub progress: Option<Progress>,
}
