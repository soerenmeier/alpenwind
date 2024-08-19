use chuchi_postgres::{time::DateTime, UniqueId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
	None,
	Insert,
	Update,
	Remove,
}

impl Change {
	pub fn set_update(&mut self, changed: bool) {
		if changed {
			*self = Change::Update;
		} else {
			*self = Change::None;
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
	// if change Inserted, this means an id was newly generated
	pub id: UniqueId,
	pub tmdb_id: Option<i64>,
	pub name: String,
	pub original_name: Option<String>,
	pub description: Option<String>,
	pub poster: Option<String>,
	pub background: Option<String>,
	pub rating: Option<f32>,
	pub data: EntryData,
	pub updated_on: DateTime,
	pub genres: Vec<UniqueId>,
	// data change should not be tracked here
	pub change: Change,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryData {
	Movie(Movie),
	Series(Series),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Movie {
	pub duration: u32,
	pub year: u16,
	pub change: Change,
	pub progress: Option<Progress>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
	pub percent: f32,
	pub updated_on: DateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Series {
	// are ordered but might have gaps
	pub seasons: Vec<Season>,
	// a season change should not affect this change
	pub change: Change,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Season {
	// if change Inserted, this means an id was newly generated
	pub id: UniqueId,
	pub season: u16,
	pub name: Option<String>,
	pub original_name: Option<String>,
	// are ordered but might have gaps
	pub episodes: Vec<Episode>,
	// a episode change should not affect this change
	pub change: Change,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Episode {
	// if change Inserted, this means an id was newly generated
	pub id: UniqueId,
	pub episode: u16,
	pub name: String,
	pub original_name: Option<String>,
	pub updated_on: DateTime,
	pub change: Change,
	pub progress: Option<Progress>,
}
