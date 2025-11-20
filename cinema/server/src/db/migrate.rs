use std::collections::HashMap;

use chuchi_postgres::Database;
use chuchi_postgres::UniqueId;

use super::old as ol;
use crate::db as ne;

pub async fn run_migration(
	old: &ol::CinemaDb,
	new: &ne::CinemaDb,
	db: &Database,
) {
	let conn = db.get().await.unwrap();
	let old = old.with_conn(conn.connection());
	let new = new.with_conn(conn.connection());

	// load all data to make sure everything works
	let entries = old.all_entries().await.unwrap();
	let progress = old.all_progress().await.unwrap();

	eprintln!("Migrating {} entries", entries.len());
	eprintln!("Migrating {} progress records", progress.len());

	// Map to track episode IDs: (entry_id, season_idx, episode_idx) -> episode_id
	let mut episode_map: HashMap<(UniqueId, usize, usize), UniqueId> =
		HashMap::new();

	// First pass: migrate all entries, seasons, and episodes
	for entry in &entries {
		match &entry.data.0 {
			ol::EntryData::Movie { year } => {
				// Create movie entry
				let n_entry = ne::Entry {
					id: entry.id,
					tmdb_id: None,
					kind: 0, // Movie
					name: entry.name.clone(),
					original_name: None,
					description: None,
					poster: None,
					background: None,
					rating: None,
					duration: None,
					first_publication: Some(*year as i16),
					created_on: entry.updated_on.clone(),
					last_updated: entry.updated_on.clone(),
				};

				new.insert_entry(&n_entry).await.unwrap();
			}
			ol::EntryData::Series { seasons } => {
				// Create series entry
				let n_entry = ne::Entry {
					id: entry.id,
					tmdb_id: None,
					kind: 1, // Series
					name: entry.name.clone(),
					original_name: None,
					description: None,
					poster: None,
					background: None,
					rating: None,
					duration: None,
					first_publication: None,
					created_on: entry.updated_on.clone(),
					last_updated: entry.updated_on.clone(),
				};

				new.insert_entry(&n_entry).await.unwrap();

				// Create seasons and episodes
				for (season_idx, season) in seasons.iter().enumerate() {
					let season_id = UniqueId::new();
					let n_season = ne::Season {
						id: season_id,
						entry_id: entry.id,
						season: season_idx as i16 + 1,
						name: season.name.clone(),
						original_name: None,
						created_on: entry.updated_on.clone(),
					};

					new.insert_season(&n_season).await.unwrap();

					// Create episodes
					for (episode_idx, episode) in
						season.episodes.iter().enumerate()
					{
						let episode_id = UniqueId::new();
						let n_episode = ne::Episode {
							id: episode_id,
							season_id,
							episode: episode_idx as i16 + 1,
							name: episode.name.clone(),
							original_name: None,
							publication_year: None,
							created_on: episode.updated_on.clone(),
							description: None,
							duration: None,
						};

						new.insert_episode(&n_episode).await.unwrap();

						// Store the mapping
						episode_map.insert(
							(entry.id, season_idx, episode_idx),
							episode_id,
						);
					}
				}
			}
		}
	}

	// Second pass: migrate progress
	for prog in progress {
		match &prog.data.0 {
			ol::EntryProgressData::Movie {
				progress: movie_prog,
			} => {
				let n_progress = ne::Progress {
					entry_id: Some(prog.entry_id),
					episode_id: None,
					user_id: prog.user_id,
					progress: movie_prog.percent,
					created_on: prog.updated_on.clone(),
					updated_on: prog.updated_on.clone(),
					last_watch: if movie_prog.percent >= 0.9 {
						Some(prog.updated_on.clone())
					} else {
						None
					},
				};

				new.update_progress(n_progress).await.unwrap();
			}
			ol::EntryProgressData::Series { seasons } => {
				// Get the entry to ensure it's a series
				let entry = entries
					.iter()
					.find(|e| e.id == prog.entry_id)
					.expect("Entry not found for progress");

				if let ol::EntryData::Series { .. } = &entry.data.0 {
					for (season_idx, season_progress) in
						seasons.iter().enumerate()
					{
						for (episode_idx, episode_progress) in
							season_progress.iter().enumerate()
						{
							if let Some(ep_prog) = episode_progress {
								// Look up the episode ID from our mapping
								if let Some(&episode_id) = episode_map.get(&(
									prog.entry_id,
									season_idx,
									episode_idx,
								)) {
									let n_progress = ne::Progress {
										entry_id: None,
										episode_id: Some(episode_id),
										user_id: prog.user_id,
										progress: ep_prog.progress.percent,
										created_on: ep_prog.updated_on.clone(),
										updated_on: ep_prog.updated_on.clone(),
										last_watch: if ep_prog.progress.percent
											>= 0.9
										{
											Some(ep_prog.updated_on.clone())
										} else {
											None
										},
									};

									new.update_progress(n_progress)
										.await
										.unwrap();
								} else {
									eprintln!(
										"Warning: Could not find episode ID for entry {} season {} episode {}",
										prog.entry_id, season_idx, episode_idx
									);
								}
							}
						}
					}
				}
			}
		}
	}

	eprintln!("\nMigration completed!");
	eprintln!("Total entries migrated: {}", entries.len());
	eprintln!("Total episodes tracked: {}", episode_map.len());
}
