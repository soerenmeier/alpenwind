use crate::data::Change;

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

const MIGRATIONS: &[(&str, &str)] = migration_files!("cinema-create");

#[derive(Resource)]
pub struct CinemaDb {
	entries: Table,
	seasons: Table,
	episodes: Table,
	media_files: Table,
	entry_genres: Table,
	progress: Table,
}

impl CinemaDb {
	pub async fn new(db: &Database) -> Self {
		let this = Self {
			entries: Table::new("cinema_entries"),
			seasons: Table::new("cinema_seasons"),
			episodes: Table::new("cinema_episodes"),
			media_files: Table::new("cinema_media_files"),
			entry_genres: Table::new("cinema_entry_genres"),
			progress: Table::new("cinema_progress"),
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
			entries: self.entries.with_conn(conn.clone()),
			seasons: self.seasons.with_conn(conn.clone()),
			episodes: self.episodes.with_conn(conn.clone()),
			media_files: self.media_files.with_conn(conn.clone()),
			entry_genres: self.entry_genres.with_conn(conn.clone()),
			progress: self.progress.with_conn(conn.clone()),
		}
	}
}

pub struct CinemaDbWithConn<'a> {
	entries: TableWithConn<'a>,
	seasons: TableWithConn<'a>,
	episodes: TableWithConn<'a>,
	media_files: TableWithConn<'a>,
	entry_genres: TableWithConn<'a>,
	progress: TableWithConn<'a>,
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Entry {
	id: UniqueId,
	tmdb_id: Option<i64>,
	kind: i8,
	name: String,
	original_name: Option<String>,
	description: Option<String>,
	poster: Option<String>,
	background: Option<String>,
	rating: Option<f32>,
	duration: Option<i32>,
	first_publication: i16,
	created_on: DateTime,
	last_updated: DateTime,
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Season {
	id: UniqueId,
	entry_id: UniqueId,
	season: i16,
	name: Option<String>,
	original_name: Option<String>,
	created_on: DateTime,
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct Episode {
	id: UniqueId,
	season_id: UniqueId,
	episode: i16,
	name: String,
	original_name: Option<String>,
	publication_year: Option<i16>,
	created_on: DateTime,
	description: Option<String>,
	duration: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct EntryGenre {
	entry_id: UniqueId,
	genre_id: UniqueId,
}

#[derive(Debug, Clone, PartialEq, FromRow, ToRow)]
pub struct Progress {
	pub entry_id: Option<UniqueId>,
	pub episode_id: Option<UniqueId>,
	pub user_id: UniqueId,
	pub progress: f32,
	pub created_on: DateTime,
	pub updated_on: DateTime,
	// the time this entry was last completelly watched
	pub last_watch: Option<DateTime>,
}

fn try_into_def<T, O>(v: T, def: O) -> O
where
	T: TryInto<O>,
{
	v.try_into().unwrap_or(def)
}

fn into_data(
	entries: Vec<Entry>,
	seasons: Vec<Season>,
	episodes: Vec<Episode>,
	entry_genres: Vec<EntryGenre>,
	progress: Vec<Progress>,
) -> HashMap<UniqueId, data::Entry> {
	let mut entries: HashMap<_, _> = entries
		.into_iter()
		.map(|e| {
			(
				e.id,
				data::Entry {
					id: e.id,
					tmdb_id: e.tmdb_id,
					name: e.name,
					original_name: e.original_name,
					description: e.description,
					poster: e.poster,
					background: e.background,
					rating: e.rating,
					updated_on: e.last_updated,
					genres: vec![],
					data: match e.kind {
						0 => data::EntryData::Movie(data::Movie {
							duration: e
								.duration
								.and_then(|u| u.try_into().ok())
								.unwrap_or(0),
							year: try_into_def(e.first_publication, 0),
							change: Change::None,
							progress: None,
						}),
						1 => data::EntryData::Series(data::Series {
							seasons: vec![],
							change: Change::None,
						}),
						_ => unreachable!(),
					},
					change: Change::None,
				},
			)
		})
		.collect();

	let mut seasons: HashMap<_, _> = seasons
		.into_iter()
		.map(|s| {
			(
				s.id,
				(
					s.entry_id,
					data::Season {
						id: s.id,
						season: s.season.try_into().unwrap(),
						name: s.name,
						original_name: s.original_name,
						episodes: vec![],
						change: Change::None,
					},
				),
			)
		})
		.collect();

	let mut episodes: HashMap<_, _> = episodes
		.into_iter()
		.map(|e| {
			(
				e.id,
				(
					e.season_id,
					data::Episode {
						id: e.id,
						episode: e.episode.try_into().unwrap(),
						name: e.name,
						original_name: e.original_name,
						// year: e.publication_year,
						// duration: e.duration.unwrap_or(0),
						// description: e.description,
						updated_on: e.created_on,
						change: Change::None,
						progress: None,
					},
				),
			)
		})
		.collect();

	for progress in progress {
		let data_progress = data::Progress {
			percent: progress.progress,
			updated_on: progress.updated_on,
		};

		match (progress.entry_id, progress.episode_id) {
			(Some(entry_id), None) => {
				let entry = entries.get_mut(&entry_id).unwrap();
				match &mut entry.data {
					data::EntryData::Movie(movie) => {
						movie.progress = Some(data_progress);
					}
					_ => unreachable!(),
				}
			}
			(None, Some(episode_id)) => {
				let (_, episode) = episodes.get_mut(&episode_id).unwrap();
				episode.progress = Some(data_progress);
			}
			_ => unreachable!(),
		}
	}

	for (_, (season_id, episode)) in episodes {
		let (_, season) = seasons.get_mut(&season_id).unwrap();
		season.episodes.push(episode);
	}

	for (entry_id, season) in seasons.values() {
		let entry = entries.get_mut(&entry_id).unwrap();
		match &mut entry.data {
			data::EntryData::Series(series) => {
				series.seasons.push(season.clone());
			}
			_ => unreachable!(),
		}
	}

	for entry_genre in entry_genres {
		let entry = entries.get_mut(&entry_genre.entry_id).unwrap();
		entry.genres.push(entry_genre.genre_id);
	}

	entries
}

impl CinemaDbWithConn<'_> {
	pub async fn all(&self) -> Result<HashMap<UniqueId, data::Entry>> {
		let (entries, seasons, episodes, entry_genres) = tokio::try_join!(
			self.entries.select::<Entry>(filter!()),
			self.seasons.select::<Season>(filter!()),
			self.episodes.select::<Episode>(filter!()),
			self.entry_genres.select::<EntryGenre>(filter!())
		)?;

		Ok(into_data(entries, seasons, episodes, entry_genres, vec![]))
	}

	pub async fn all_by_user(
		&self,
		user_id: &UniqueId,
	) -> Result<HashMap<UniqueId, data::Entry>> {
		let (entries, seasons, episodes, entry_genres, progress) = tokio::try_join!(
			self.entries.select::<Entry>(filter!()),
			self.seasons.select::<Season>(filter!()),
			self.episodes.select::<Episode>(filter!()),
			self.entry_genres.select::<EntryGenre>(filter!()),
			self.progress.select::<Progress>(filter!(user_id))
		)?;

		Ok(into_data(
			entries,
			seasons,
			episodes,
			entry_genres,
			progress,
		))
	}

	pub async fn by_id_and_user(
		&self,
		id: &UniqueId,
		user_id: &UniqueId,
	) -> Result<Option<data::Entry>> {
		let (entries, seasons, entry_genres) = tokio::try_join!(
			self.entries.select::<Entry>(filter!(id)),
			self.seasons.select::<Season>(filter!("entry_id" = id)),
			self.entry_genres
				.select::<EntryGenre>(filter!("entry_id" = id))
		)?;

		// now if there are seasons we need to query episodes
		let episodes = if !seasons.is_empty() {
			let season_ids = seasons.iter().map(|s| s.id).collect::<Vec<_>>();
			self.episodes
				.select::<Episode>(filter!("season_id" IN &season_ids))
				.await?
		} else {
			vec![]
		};

		let episode_ids = episodes.iter().map(|e| e.id).collect::<Vec<_>>();
		let progress = self
			.progress
			.select::<Progress>(
				filter!(user_id AND ("entry_id" = id OR "episode_id" IN &episode_ids)),
			)
			.await?;

		let entries =
			into_data(entries, seasons, episodes, entry_genres, progress);

		Ok(entries.into_iter().next().map(|(_, e)| e))
	}

	pub async fn movie_exists(&self, id: &UniqueId) -> Result<bool> {
		#[derive(Debug, FromRow)]
		struct EntryType {
			id: UniqueId,
			kind: i8,
		}

		let entry = self.entries.select_opt::<EntryType>(filter!(id)).await?;

		Ok(entry.map(|e| e.kind == 0).unwrap_or(false))
	}

	pub async fn episode_exists(&self, id: &UniqueId) -> Result<bool> {
		Ok(self.episodes.count("id", filter!(id)).await? > 0)
	}

	pub async fn progress_by_id_user(
		&self,
		entry_id: &Option<UniqueId>,
		episode_id: &Option<UniqueId>,
		user_id: &UniqueId,
	) -> Result<Option<Progress>> {
		let progress = self
			.progress
			.select_opt::<Progress>(
				filter!(entry_id AND episode_id AND user_id),
			)
			.await?;
		Ok(progress)
	}

	pub async fn update_progress(&self, progress: Progress) -> Result<()> {
		// let prog = EntryProgress::from_entry(entry, user_id);
		let table = self.progress.name();
		let conn = self.progress.conn();

		let mut sql = format!("INSERT INTO \"{table}\" (");
		progress.insert_columns(&mut sql);
		sql.push_str(") VALUES (");
		progress.insert_values(&mut sql);
		sql.push_str(
			") ON CONFLICT (\"entry_id\", \"episode_id\", \"user_id\") DO UPDATE SET \
				\"percent\" = excluded.percent,\
				\"updated_on\" = excluded.updated_on,\
				\"last_watch\" = excluded.last_watch\
			",
		);

		let stmt = conn.prepare_cached(&sql).await?;

		conn.execute_raw(&stmt, progress.params()).await.map(|_| ())

		// self.table_progress.execute_raw(sql, &prog.to_data()).await
	}
}

// impl Entry {
// 	pub fn from_data(e: &data::Entry) -> Self {
// 		match e {
// 			data::Entry::Movie(m) => Entry {
// 				id: m.id,
// 				name: m.name.clone(),
// 				updated_on: m.updated_on.clone(),
// 				data: Json(EntryData::Movie { year: m.year }),
// 			},
// 			data::Entry::Series(s) => Entry {
// 				id: s.id,
// 				name: s.name.clone(),
// 				updated_on: DateTime::now(),
// 				data: Json(EntryData::Series {
// 					seasons: s
// 						.seasons
// 						.iter()
// 						.map(|s| Season {
// 							name: s.name.clone(),
// 							episodes: s
// 								.episodes
// 								.iter()
// 								.map(|e| Episode {
// 									name: e.name.clone(),
// 									updated_on: e.updated_on.clone(),
// 								})
// 								.collect(),
// 						})
// 						.collect(),
// 				}),
// 			},
// 		}
// 	}

// 	/// Returns none if the progress does not match with the entry
// 	pub fn into_data(
// 		self,
// 		progress: Option<EntryProgress>,
// 	) -> Option<data::Entry> {
// 		match self.data.0 {
// 			EntryData::Movie { year } => {
// 				let movie_progress = match progress {
// 					Some(prog) => Some(prog.into_movie()?),
// 					None => None,
// 				};

// 				Some(data::Entry::Movie(data::Movie {
// 					id: self.id,
// 					name: self.name,
// 					year,
// 					updated_on: self.updated_on,
// 					progress: movie_progress,
// 				}))
// 			}
// 			EntryData::Series { seasons } => {
// 				let seasons_prog = match progress {
// 					Some(prog) => prog.into_seasons()?,
// 					None => vec![],
// 				};

// 				Some(data::Entry::Series(data::Series {
// 					id: self.id,
// 					name: self.name,
// 					seasons: seasons
// 						.into_iter()
// 						.enumerate()
// 						.map(|(idx, season)| {
// 							let episodes_prog = seasons_prog
// 								.get(idx)
// 								.map(|p| p.as_slice())
// 								.unwrap_or(&[]);

// 							data::Season {
// 								name: season.name,
// 								episodes: season
// 									.episodes
// 									.into_iter()
// 									.enumerate()
// 									.map(|(idx, episode)| data::Episode {
// 										name: episode.name,
// 										updated_on: episode.updated_on,
// 										progress: episodes_prog
// 											.get(idx)
// 											.and_then(|e| e.as_ref())
// 											.map(|e| e.to_progress()),
// 									})
// 									.collect(),
// 							}
// 						})
// 						.collect(),
// 				}))
// 			}
// 		}
// 	}
// }

// impl EntryProgress {
// 	pub fn into_movie(self) -> Option<data::Progress> {
// 		match self.data.0 {
// 			EntryProgressData::Movie { progress } => Some(data::Progress {
// 				percent: progress.percent,
// 				position: progress.position,
// 				updated_on: self.updated_on,
// 			}),
// 			_ => None,
// 		}
// 	}

// 	pub fn into_seasons(self) -> Option<Vec<Vec<Option<EpisodeProgress>>>> {
// 		match self.data.0 {
// 			EntryProgressData::Series { seasons } => Some(seasons),
// 			_ => None,
// 		}
// 	}

// 	/// updates the update_on field on the series
// 	pub fn from_entry(entry: &data::Entry, user_id: &UniqueId) -> Self {
// 		match entry {
// 			data::Entry::Movie(m) => {
// 				let prog = m.progress.as_ref().unwrap();
// 				Self {
// 					entry_id: m.id,
// 					user_id: *user_id,
// 					updated_on: prog.updated_on.clone(),
// 					data: Json(EntryProgressData::Movie {
// 						progress: Progress {
// 							percent: prog.percent,
// 							position: prog.position,
// 						},
// 					}),
// 				}
// 			}
// 			data::Entry::Series(s) => Self {
// 				entry_id: s.id,
// 				user_id: *user_id,
// 				updated_on: DateTime::now(),
// 				data: Json(EntryProgressData::Series {
// 					seasons: s
// 						.seasons
// 						.iter()
// 						.map(|s| {
// 							s.episodes
// 								.iter()
// 								.map(|e| {
// 									e.progress.as_ref().map(|prog| {
// 										EpisodeProgress {
// 											progress: Progress {
// 												percent: prog.percent,
// 												position: prog.position,
// 											},
// 											updated_on: prog.updated_on.clone(),
// 										}
// 									})
// 								})
// 								.collect()
// 						})
// 						.collect(),
// 				}),
// 			},
// 		}
// 	}
// }

// impl EpisodeProgress {
// 	pub fn to_progress(&self) -> data::Progress {
// 		data::Progress {
// 			percent: self.progress.percent,
// 			position: self.progress.position,
// 			updated_on: self.updated_on.clone(),
// 		}
// 	}
// }
