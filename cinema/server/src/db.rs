use super::data;

use std::collections::HashMap;
use std::fmt::Write;

use postgres::{Result, Table, Database, UniqueId, TableTempl};
use postgres::{whr, impl_json_col_type, try2};
use postgres::time::DateTime;
use postgres::query::SqlBuilder;
use postgres::table::TableTemplate;

use serde::{Serialize, Deserialize};


#[derive(Debug, TableTempl)]
struct Entry {
	#[index(primary)]
	id: UniqueId,
	name: String,
	updated_on: DateTime,
	data: EntryData
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntryData {
	Movie {
		year: u32
	},
	Series {
		seasons: Vec<Season>
	}
}

impl_json_col_type!(EntryData);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Season {
	name: Option<String>,
	episodes: Vec<Episode>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Episode {
	name: String,
	updated_on: DateTime
}

#[derive(Debug, TableTempl)]
struct EntryProgress {
	#[index(primary)]
	entry_id: UniqueId,
	#[index(primary)]
	user_id: UniqueId,
	updated_on: DateTime,
	data: EntryProgressData
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntryProgressData {
	Movie {
		progress: Progress
	},
	Series {
		seasons: Vec<Vec<Option<EpisodeProgress>>>
	}
}

impl_json_col_type!(EntryProgressData);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpisodeProgress {
	progress: Progress,
	updated_on: DateTime
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
	percent: f32,
	position: f32
}


pub struct CinemaDb {
	table: Table<Entry>,
	table_progress: Table<EntryProgress>
}

impl CinemaDb {
	pub async fn new(db: &Database) -> Self {
		Self {
			table: db.table("cinema").create().await,
			table_progress: db.table("cinema_progress").create().await
		}
	}

	pub async fn all(&self) -> Result<Vec<data::Entry>> {
		let entries = self.table.find_all().await?;
		Ok(entries.into_iter().map(|e| e.into_data(None).unwrap()).collect())
	}

	pub async fn all_by_user(
		&self,
		user_id: &UniqueId
	) -> Result<Vec<data::Entry>> {
		let entries = self.table.find_all().await?;
		let progress = self.table_progress.find_many(whr!(user_id)).await?;

		let mut progress: HashMap<_, _> = progress.into_iter()
			.map(|p| (p.entry_id, p))
			.collect();

		Ok(entries.into_iter().filter_map(|e| {
			let id = e.id;
			let s = e.into_data(progress.remove(&id));
			if s.is_none() {
				eprintln!(
					"progress data incosistent: entry {id} user {user_id}"
				);
			}

			s
		}).collect())
	}

	pub async fn by_id_and_user(
		&self,
		id: &UniqueId,
		user_id: &UniqueId
	) -> Result<Option<data::Entry>> {
		let entry = try2!(self.table.find_one(whr!(id)).await?);
		let progress = self.table_progress.find_one(
			whr!("entry_id"=id AND user_id)
		).await?;

		Ok(entry.into_data(progress))
	}

	pub async fn update_progress(
		&self,
		entry: &data::Entry,
		user_id: &UniqueId
	) -> Result<()> {
		let prog = EntryProgress::from_entry(entry, user_id);

		let mut names = String::new();
		let mut values = String::new();

		for (idx, col) in self.table_progress.info().data().iter().enumerate() {
			if idx != 0 {
				names.push(',');
				values.push(',');
			}

			write!(names, "\"{}\"", col.name).unwrap();
			write!(values, "${}", idx + 1).unwrap();
		}

		let mut sql = SqlBuilder::new();
		sql.no_space("INSERT INTO \"");
		sql.no_space(self.table_progress.name());
		sql.no_space("\" (");
		sql.no_space(names);
		sql.no_space(") VALUES (");
		sql.no_space(values);
		sql.no_space(") ON CONFLICT (\"entry_id\", \"user_id\") DO UPDATE SET \
			\"updated_on\" = excluded.updated_on,\
			\"data\" = excluded.data\
		");

		self.table_progress.execute_raw(sql, &prog.to_data()).await
	}

	pub async fn insert_data(&self, entry: &data::Entry) -> Result<()> {
		let e = Entry::from_data(entry);

		self.table.insert_one(&e).await
	}

	pub async fn update_data(&self, entry: &data::Entry) -> Result<()> {
		let e = Entry::from_data(entry);
		let id = &e.id;

		self.table.update_full(whr!(id), &e).await
	}

	pub async fn delete_by_id(&self, id: &UniqueId) -> Result<()> {
		self.table.delete(whr!(id)).await?;
		self.table_progress.delete(whr!("entry_id" = id)).await
	}
}

/*

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntryData {
	Movie {
		year: u32
	},
	Series {
		seasons: Vec<Season>
	}
}

impl_json_col_type!(EntryData);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Season {
	name: Option<String>,
	episodes: Vec<Episode>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Episode {
	name: String,
	updated_on: DateTime
}
*/

impl Entry {
	pub fn from_data(e: &data::Entry) -> Self {
		match e {
			data::Entry::Movie(m) => {
				Entry {
					id: m.id,
					name: m.name.clone(),
					updated_on: m.updated_on.clone(),
					data: EntryData::Movie {
						year: m.year
					}
				}
			},
			data::Entry::Series(s) => {
				Entry {
					id: s.id,
					name: s.name.clone(),
					updated_on: DateTime::now(),
					data: EntryData::Series {
						seasons: s.seasons.iter().map(|s| Season {
							name: s.name.clone(),
							episodes: s.episodes.iter().map(|e| Episode {
								name: e.name.clone(),
								updated_on: e.updated_on.clone()
							}).collect()
						}).collect()
					}
				}
			}
		}
	}

	/// Returns none if the progress does not match with the entry
	pub fn into_data(
		self,
		progress: Option<EntryProgress>
	) -> Option<data::Entry> {
		match self.data {
			EntryData::Movie { year } => {
				let movie_progress = match progress {
					Some(prog) => Some(prog.into_movie()?),
					None => None
				};

				Some(data::Entry::Movie(data::Movie {
					id: self.id,
					name: self.name,
					year: year,
					updated_on: self.updated_on,
					progress: movie_progress
				}))
			},
			EntryData::Series { seasons } => {
				let seasons_prog = match progress {
					Some(prog) => prog.into_seasons()?,
					None => vec![]
				};

				Some(data::Entry::Series(data::Series {
					id: self.id,
					name: self.name,
					seasons: seasons.into_iter().enumerate()
						.map(|(idx, season)| {
							let episodes_prog = seasons_prog.get(idx)
								.map(|p| p.as_slice())
								.unwrap_or(&[]);

							data::Season {
								name: season.name,
								episodes: season.episodes.into_iter()
									.enumerate()
									.map(|(idx, episode)| {
										data::Episode {
											name: episode.name,
											updated_on: episode.updated_on,
											progress: episodes_prog.get(idx)
												.and_then(|e| e.as_ref())
												.map(|e| e.to_progress())
										}
									}).collect()
							}
						}).collect()
				}))
			}
		}
	}
}

impl EntryProgress {
	pub fn into_movie(self) -> Option<data::Progress> {
		match self.data {
			EntryProgressData::Movie { progress } => Some(data::Progress {
				percent: progress.percent,
				position: progress.position,
				updated_on: self.updated_on
			}),
			_ => None
		}
	}

	pub fn into_seasons(self) -> Option<Vec<Vec<Option<EpisodeProgress>>>> {
		match self.data {
			EntryProgressData::Series { seasons } => Some(seasons),
			_ => None
		}
	}

	/// updates the update_on field on the series
	pub fn from_entry(entry: &data::Entry, user_id: &UniqueId) -> Self {
		match entry {
			data::Entry::Movie(m) => {
				let prog = m.progress.as_ref().unwrap();
				Self {
					entry_id: m.id,
					user_id: *user_id,
					updated_on: prog.updated_on.clone(),
					data: EntryProgressData::Movie {
						progress: Progress {
							percent: prog.percent,
							position: prog.position
						}
					}
				}
			},
			data::Entry::Series(s) => {
				Self {
					entry_id: s.id,
					user_id: *user_id,
					updated_on: DateTime::now(),
					data: EntryProgressData::Series {
						seasons: s.seasons.iter()
							.map(|s| {
								s.episodes.iter().map(|e| {
									e.progress.as_ref()
										.map(|prog| EpisodeProgress {
										progress: Progress {
											percent: prog.percent,
											position: prog.position
										},
										updated_on: prog.updated_on.clone()
									})
								}).collect()
							}).collect()
					}
				}
			}
		}
	}
}

impl EpisodeProgress {
	pub fn to_progress(&self) -> data::Progress {
		data::Progress {
			percent: self.progress.percent,
			position: self.progress.position,
			updated_on: self.updated_on.clone()
		}
	}
}