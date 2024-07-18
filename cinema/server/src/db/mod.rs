use super::data;

use std::collections::HashMap;

use chuchi::Resource;
use chuchi_postgres::json::Json;
use chuchi_postgres::row::ToRow;
use chuchi_postgres::table::table::TableWithConn;
use chuchi_postgres::table::Table;
use chuchi_postgres::time::DateTime;
use chuchi_postgres::{filter, try2, whr, Connection, FromRow, ToRow};
use chuchi_postgres::{Database, Result, UniqueId};

use core_lib::migration_files;
use serde::{Deserialize, Serialize};

/// Todo redo without TableTempl
#[derive(Debug, FromRow, ToRow)]
struct Entry {
	id: UniqueId,
	name: String,
	updated_on: DateTime,
	data: Json<EntryData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntryData {
	Movie { year: u32 },
	Series { seasons: Vec<Season> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Season {
	name: Option<String>,
	episodes: Vec<Episode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Episode {
	name: String,
	updated_on: DateTime,
}

#[derive(Debug, FromRow, ToRow)]
struct EntryProgress {
	entry_id: UniqueId,
	user_id: UniqueId,
	updated_on: DateTime,
	data: Json<EntryProgressData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum EntryProgressData {
	Movie {
		progress: Progress,
	},
	Series {
		seasons: Vec<Vec<Option<EpisodeProgress>>>,
	},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpisodeProgress {
	progress: Progress,
	updated_on: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
	percent: f32,
	position: f32,
}

const MIGRATIONS: &[(&str, &str)] = migration_files!("cinema-create");

#[derive(Resource)]
pub struct CinemaDb {
	table: Table,
	table_progress: Table,
}

impl CinemaDb {
	pub async fn new(db: &Database) -> Self {
		let this = Self {
			table: Table::new("cinema"),
			table_progress: Table::new("cinema_progress"),
		};

		let migrations = db.migrations();
		let mut conn = db.get().await.unwrap();

		for (name, sql) in MIGRATIONS {
			migrations
				.add(&mut conn, name, sql)
				.await
				.expect("failed to run migration");
		}

		this
	}

	pub fn with_conn<'a>(
		&'a self,
		conn: Connection<'a>,
	) -> CinemaDbWithConn<'a> {
		CinemaDbWithConn {
			table: self.table.with_conn(conn.clone()),
			table_progress: self.table_progress.with_conn(conn),
		}
	}
}

pub struct CinemaDbWithConn<'a> {
	table: TableWithConn<'a>,
	table_progress: TableWithConn<'a>,
}

impl CinemaDbWithConn<'_> {
	pub async fn all(&self) -> Result<Vec<data::Entry>> {
		let entries = self.table.select::<Entry>(filter!()).await?;
		Ok(entries
			.into_iter()
			.map(|e| e.into_data(None).unwrap())
			.collect())
	}

	pub async fn all_by_user(
		&self,
		user_id: &UniqueId,
	) -> Result<Vec<data::Entry>> {
		let entries = self.table.select::<Entry>(filter!()).await?;
		let progress = self
			.table_progress
			.select::<EntryProgress>(filter!(user_id))
			.await?;

		let mut progress: HashMap<_, _> =
			progress.into_iter().map(|p| (p.entry_id, p)).collect();

		Ok(entries
			.into_iter()
			.filter_map(|e| {
				let id = e.id;
				let s = e.into_data(progress.remove(&id));
				if s.is_none() {
					eprintln!(
						"progress data incosistent: entry {id} user {user_id}"
					);
				}

				s
			})
			.collect())
	}

	pub async fn by_id_and_user(
		&self,
		id: &UniqueId,
		user_id: &UniqueId,
	) -> Result<Option<data::Entry>> {
		let entry = try2!(self.table.select_opt::<Entry>(filter!(id)).await?);
		let progress = self
			.table_progress
			.select_opt::<EntryProgress>(filter!("entry_id"=id AND user_id))
			.await?;

		Ok(entry.into_data(progress))
	}

	pub async fn update_progress(
		&self,
		entry: &data::Entry,
		user_id: &UniqueId,
	) -> Result<()> {
		let prog = EntryProgress::from_entry(entry, user_id);
		let table = self.table_progress.name();
		let conn = self.table_progress.conn();

		let mut sql = format!("INSERT INTO \"{table}\" (");
		prog.insert_columns(&mut sql);
		sql.push_str(") VALUES (");
		prog.insert_values(&mut sql);
		sql.push_str(
			") ON CONFLICT (\"entry_id\", \"user_id\") DO UPDATE SET \
				\"updated_on\" = excluded.updated_on,\
				\"data\" = excluded.data\
			",
		);

		let stmt = conn.prepare_cached(&sql).await?;

		conn.execute_raw(&stmt, prog.params()).await.map(|_| ())

		// self.table_progress.execute_raw(sql, &prog.to_data()).await
	}

	pub async fn insert_data(&self, entry: &data::Entry) -> Result<()> {
		let e = Entry::from_data(entry);

		self.table.insert(&e).await
	}

	pub async fn update_data(&self, entry: &data::Entry) -> Result<()> {
		let e = Entry::from_data(entry);
		let id = &e.id;

		self.table.update(&e, whr!(id)).await
	}

	pub async fn delete_by_id(&self, id: &UniqueId) -> Result<()> {
		self.table.delete(whr!(id)).await?;
		self.table_progress.delete(whr!("entry_id" = id)).await
	}
}

impl Entry {
	pub fn from_data(e: &data::Entry) -> Self {
		match e {
			data::Entry::Movie(m) => Entry {
				id: m.id,
				name: m.name.clone(),
				updated_on: m.updated_on.clone(),
				data: Json(EntryData::Movie { year: m.year }),
			},
			data::Entry::Series(s) => Entry {
				id: s.id,
				name: s.name.clone(),
				updated_on: DateTime::now(),
				data: Json(EntryData::Series {
					seasons: s
						.seasons
						.iter()
						.map(|s| Season {
							name: s.name.clone(),
							episodes: s
								.episodes
								.iter()
								.map(|e| Episode {
									name: e.name.clone(),
									updated_on: e.updated_on.clone(),
								})
								.collect(),
						})
						.collect(),
				}),
			},
		}
	}

	/// Returns none if the progress does not match with the entry
	pub fn into_data(
		self,
		progress: Option<EntryProgress>,
	) -> Option<data::Entry> {
		match self.data.0 {
			EntryData::Movie { year } => {
				let movie_progress = match progress {
					Some(prog) => Some(prog.into_movie()?),
					None => None,
				};

				Some(data::Entry::Movie(data::Movie {
					id: self.id,
					name: self.name,
					year,
					updated_on: self.updated_on,
					progress: movie_progress,
				}))
			}
			EntryData::Series { seasons } => {
				let seasons_prog = match progress {
					Some(prog) => prog.into_seasons()?,
					None => vec![],
				};

				Some(data::Entry::Series(data::Series {
					id: self.id,
					name: self.name,
					seasons: seasons
						.into_iter()
						.enumerate()
						.map(|(idx, season)| {
							let episodes_prog = seasons_prog
								.get(idx)
								.map(|p| p.as_slice())
								.unwrap_or(&[]);

							data::Season {
								name: season.name,
								episodes: season
									.episodes
									.into_iter()
									.enumerate()
									.map(|(idx, episode)| data::Episode {
										name: episode.name,
										updated_on: episode.updated_on,
										progress: episodes_prog
											.get(idx)
											.and_then(|e| e.as_ref())
											.map(|e| e.to_progress()),
									})
									.collect(),
							}
						})
						.collect(),
				}))
			}
		}
	}
}

impl EntryProgress {
	pub fn into_movie(self) -> Option<data::Progress> {
		match self.data.0 {
			EntryProgressData::Movie { progress } => Some(data::Progress {
				percent: progress.percent,
				position: progress.position,
				updated_on: self.updated_on,
			}),
			_ => None,
		}
	}

	pub fn into_seasons(self) -> Option<Vec<Vec<Option<EpisodeProgress>>>> {
		match self.data.0 {
			EntryProgressData::Series { seasons } => Some(seasons),
			_ => None,
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
					data: Json(EntryProgressData::Movie {
						progress: Progress {
							percent: prog.percent,
							position: prog.position,
						},
					}),
				}
			}
			data::Entry::Series(s) => Self {
				entry_id: s.id,
				user_id: *user_id,
				updated_on: DateTime::now(),
				data: Json(EntryProgressData::Series {
					seasons: s
						.seasons
						.iter()
						.map(|s| {
							s.episodes
								.iter()
								.map(|e| {
									e.progress.as_ref().map(|prog| {
										EpisodeProgress {
											progress: Progress {
												percent: prog.percent,
												position: prog.position,
											},
											updated_on: prog.updated_on.clone(),
										}
									})
								})
								.collect()
						})
						.collect(),
				}),
			},
		}
	}
}

impl EpisodeProgress {
	pub fn to_progress(&self) -> data::Progress {
		data::Progress {
			percent: self.progress.percent,
			position: self.progress.position,
			updated_on: self.updated_on.clone(),
		}
	}
}
