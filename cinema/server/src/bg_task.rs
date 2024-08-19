use crate::data::{Change, Entry, EntryData};
use crate::db::{self, CinemaDb, CinemaDbWithConn};
use crate::error::{Error, Result};
use crate::fs::changes_from_fs;
use crate::CinemaConf;

use chuchi::resources::Resources;
use chuchi_postgres::Database;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

use core_lib::server::OnTerminate;
use tracing::info;

#[cfg(debug_assertions)]
const REFRESH_EVERY: Duration = Duration::from_secs(1 * 60);
#[cfg(not(debug_assertions))]
const REFRESH_EVERY: Duration = Duration::from_secs(5 * 60);

pub(crate) fn bg_task(
	data: Resources,
	cfg: CinemaConf,
	mut on_terminate: OnTerminate,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let mut intv = time::interval(REFRESH_EVERY);
		let cinema: &CinemaDb = data.get().unwrap();
		let db: &Database = data.get().unwrap();

		let terminate = on_terminate.on_terminate();
		tokio::pin!(terminate);
		loop {
			tokio::select! {
				_ = intv.tick() => {},
				_ = &mut terminate => return
			}

			if let Err(e) = task_tick(db, cinema, &cfg).await {
				eprintln!("failed to update cinema {e:?}");
			}
		}
	})
}

async fn task_tick(
	db: &Database,
	cinema: &CinemaDb,
	cfg: &CinemaConf,
) -> Result<()> {
	let mut conn =
		db.get().await.map_err(|e| Error::Internal(e.to_string()))?;
	let trans = conn.transaction().await?;
	let cinema = cinema.with_conn(trans.connection());
	let entries = cinema.all().await?;

	let changes = changes_from_fs(&entries, &cfg)
		.await
		.map_err(|e| Error::Internal(e.to_string()))?;

	// todo(thierry): modify data from some movie db and
	// add change: Change::Updated or Inserted as needed

	for change in changes {
		info!("applying change: {:?}", change);
		apply_change(&cinema, change).await?;
	}

	trans.commit().await?;

	Ok(())
}

async fn apply_change(db: &CinemaDbWithConn<'_>, entry: Entry) -> Result<()> {
	match entry.change {
		Change::Insert | Change::Update => {
			// create a db entry
			let mut db_entry = db::Entry {
				id: entry.id,
				tmdb_id: entry.tmdb_id,
				kind: 0,
				name: entry.name,
				original_name: entry.original_name,
				description: entry.description,
				poster: entry.poster,
				background: entry.background,
				rating: entry.rating,
				duration: None,
				first_publication: None,
				created_on: entry.created_on,
				last_updated: entry.updated_on,
			};

			match &entry.data {
				EntryData::Movie(movie) => {
					db_entry.kind = 0;
					db_entry.duration = movie.duration.map(|d| d as i32);
					db_entry.first_publication = Some(movie.year as i16);
				}
				EntryData::Series(_) => {
					db_entry.kind = 1;
					// todo: first_publication might be added later
				}
			}

			match entry.change {
				Change::Insert => {
					db.insert_entry(&db_entry).await?;
				}
				Change::Update => {
					db.update_entry(&db_entry).await?;
				}
				_ => unreachable!(),
			}
		}
		Change::Remove => {
			db.delete_entry(&entry.id).await?;
			// this should have delete all descendants
			return Ok(());
		}
		_ => {}
	}

	// now the entry is updated
	// maybe we need to update seasons and episodes
	let EntryData::Series(series) = entry.data else {
		return Ok(());
	};

	for season in series.seasons {
		match season.change {
			Change::Insert | Change::Update => {
				let db_season = db::Season {
					id: season.id,
					entry_id: entry.id,
					season: season.season as i16,
					name: season.name,
					original_name: season.original_name,
					created_on: season.created_on,
				};

				match season.change {
					Change::Insert => {
						db.insert_season(&db_season).await?;
					}
					Change::Update => {
						db.update_season(&db_season).await?;
					}
					_ => unreachable!(),
				}
			}
			Change::Remove => {
				db.delete_season(&season.id).await?;
				// this should have delete all descendants
				continue;
			}
			_ => {}
		}

		for episode in season.episodes {
			match episode.change {
				Change::Insert | Change::Update => {
					let db_episode = db::Episode {
						id: episode.id,
						season_id: season.id,
						episode: episode.episode as i16,
						name: episode.name,
						original_name: episode.original_name,
						publication_year: episode.year.map(|p| p as i16),
						created_on: episode.created_on,
						description: episode.description,
						duration: episode.duration.map(|d| d as i32),
					};

					match episode.change {
						Change::Insert => {
							db.insert_episode(&db_episode).await?;
						}
						Change::Update => {
							db.update_episode(&db_episode).await?;
						}
						_ => unreachable!(),
					}
				}
				Change::Remove => {
					db.delete_episode(&episode.id).await?;
					continue;
				}
				_ => {}
			}
		}
	}

	Ok(())
}
