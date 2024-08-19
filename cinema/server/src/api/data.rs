use crate::data;

use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(tag = "kind")]
pub enum EntryData {
	Movie(Movie),
	Series(Series),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
	pub duration: Option<u32>,
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

impl From<data::Entry> for Entry {
	fn from(entry: data::Entry) -> Self {
		Self {
			id: entry.id,
			name: entry.name,
			original_name: entry.original_name,
			description: entry.description,
			rating: entry.rating,
			data: entry.data.into(),
			updated_on: entry.updated_on,
			genres: entry.genres,
		}
	}
}

impl From<data::EntryData> for EntryData {
	fn from(data: data::EntryData) -> Self {
		match data {
			data::EntryData::Movie(movie) => Self::Movie(movie.into()),
			data::EntryData::Series(series) => Self::Series(series.into()),
		}
	}
}

impl From<data::Movie> for Movie {
	fn from(movie: data::Movie) -> Self {
		Self {
			duration: movie.duration,
			year: movie.year,
			progress: movie.progress.map(|progress| progress.into()),
		}
	}
}

impl From<data::Series> for Series {
	fn from(series: data::Series) -> Self {
		Self {
			seasons: series
				.seasons
				.into_iter()
				.map(|season| season.into())
				.collect(),
		}
	}
}

impl From<data::Season> for Season {
	fn from(season: data::Season) -> Self {
		Self {
			id: season.id,
			season: season.season,
			name: season.name,
			original_name: season.original_name,
			episodes: season
				.episodes
				.into_iter()
				.map(|episode| episode.into())
				.collect(),
		}
	}
}

impl From<data::Episode> for Episode {
	fn from(episode: data::Episode) -> Self {
		Self {
			id: episode.id,
			episode: episode.episode,
			name: episode.name,
			original_name: episode.original_name,
			updated_on: episode.created_on,
			progress: episode.progress.map(|progress| progress.into()),
		}
	}
}

impl From<data::Progress> for Progress {
	fn from(progress: data::Progress) -> Self {
		Self {
			percent: progress.percent,
			updated_on: progress.updated_on,
		}
	}
}
