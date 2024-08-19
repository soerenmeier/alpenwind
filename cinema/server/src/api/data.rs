use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
	pub id: UniqueId,
	pub name: String,
	pub original_name: Option<String>,
	pub description: Option<String>,
	pub rating: Option<f32>,
	pub data: EntryData,
	pub updated_on: DateTime,
	pub genres: Vec<UniqueId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryData {
	Movie(Movie),
	Series(Series),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
	pub duration: u32,
	pub year: u16,
	pub progress: Option<Progress>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
	pub percent: f32,
	pub updated_on: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
	// are ordered but might have gaps
	pub seasons: Vec<Season>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
	pub id: UniqueId,
	pub season: u16,
	pub name: Option<String>,
	pub original_name: Option<String>,
	// are ordered but might have gaps
	pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
	pub id: UniqueId,
	pub episode: u16,
	pub name: String,
	pub original_name: Option<String>,
	pub updated_on: DateTime,
	pub progress: Option<Progress>,
}
