mod read;
pub(super) mod route;
mod util;

use super::data;
use crate::data::Change;
use crate::CinemaConf;

use std::collections::HashMap;
use std::time::SystemTime;
use std::{io, mem};

use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;
use util::IndexedVec;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EntryId {
	kind: EntryKind,
	name: String,
	year: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EntryKind {
	Movie,
	Series,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
	Movie {
		name: String,
		year: u16,
		created_on: SystemTime,
		duration: Option<u32>,
		poster: Option<String>,
		background: Option<String>,
	},
	Series {
		name: String,
		seasons: Vec<Season>,
		poster: Option<String>,
		background: Option<String>,
	},
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Season {
	name: Option<String>,
	season: u16,
	episodes: Vec<Episode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Episode {
	name: String,
	episode: u16,
	created_on: SystemTime,
}

macro_rules! changes {
	($changed:ident, $old:expr, $ident:ident) => {
		if $old.$ident != $ident {
			$old.$ident = $ident;
			$changed = true;
		}
	};
	($changed:ident, $old:expr, $ident:ident = $new:expr) => {
		if $old.$ident != $new {
			$old.$ident = $new;
			$changed = true;
		}
	};
	($changed:ident, $old:expr, $ident:ident, $($tt:tt)*)=> {
		changes!($changed, $old, $ident);
		changes!($changed, $old, $($tt)*);
	};
}

/// take a list of Entries from the database
/// and calculate based on the filesystem what entries changeds
pub(super) async fn changes_from_fs(
	data_entries: &HashMap<UniqueId, data::Entry>,
	cfg: &CinemaConf,
) -> io::Result<Vec<data::Entry>> {
	let fs_entries = read::entries_from_fs(cfg).await?;

	// make a copy of all entries and add an internal id to them
	let mut entries: HashMap<EntryId, data::Entry> = data_entries
		.values()
		.map(|e| {
			let mut e = e.clone();
			// mark all entries as removed and we will change them later
			e.change = Change::Remove;
			(
				match &e.data {
					data::EntryData::Movie(movie) => EntryId {
						kind: EntryKind::Movie,
						name: e.name.clone(),
						year: Some(movie.year),
					},
					data::EntryData::Series(_) => EntryId {
						kind: EntryKind::Series,
						name: e.name.clone(),
						year: None,
					},
				},
				e,
			)
		})
		.collect();

	for (id, entry) in fs_entries {
		let Some(d_entry) = entries.get_mut(&id) else {
			entries.insert(id, data::Entry::from(entry));
			// convert entry to data entry
			continue;
		};

		// now we know the entry already exists
		// let's compare them and find out what changed

		match entry {
			Entry::Movie {
				name,
				year,
				duration,
				created_on,
				poster,
				background,
			} => {
				let mut changed = false;
				let updated_on = DateTime::from_std(created_on);
				changes!(
					changed, d_entry, name, poster, background, updated_on
				);

				let data::EntryData::Movie(movie) = &mut d_entry.data else {
					unreachable!()
				};

				changes!(changed, movie, duration, year);

				d_entry.change.set_update(changed);
			}
			Entry::Series {
				name,
				seasons,
				poster,
				background,
			} => {
				let mut changed = false;
				changes!(changed, d_entry, name, poster, background);

				let data::EntryData::Series(series) = &mut d_entry.data else {
					unreachable!()
				};

				d_entry.change.set_update(changed);

				changes_seasons(&mut series.seasons, seasons);
			}
		}
	}

	Ok(entries.into_iter().map(|(_, e)| e).collect())
}

fn changes_seasons(d_seasons: &mut Vec<data::Season>, seasons: Vec<Season>) {
	// we need to split the seasons
	let mut i_seasons: IndexedVec<_> = mem::take(d_seasons)
		.into_iter()
		.map(|s| (s.season as usize, s))
		.collect();

	for season in &mut i_seasons {
		season.change = Change::Remove;
	}

	// now we need to compare the seasons
	for season in seasons {
		let Some(d_season) = i_seasons.get_mut(season.season as usize) else {
			// insert a new one
			i_seasons.set(season.season as usize, season.into());
			continue;
		};

		// now we know the season already exists
		let mut changed = false;
		changes!(changed, d_season, name = season.name);

		d_season.change.set_update(changed);

		changes_episodes(&mut d_season.episodes, season.episodes);
	}

	*d_seasons = i_seasons.into_iter().collect();
}

fn changes_episodes(
	d_episodes: &mut Vec<data::Episode>,
	episodes: Vec<Episode>,
) {
	// we need to split the episodes into an indexed vec
	// this allows us to easily find the episodes and insert new ones
	let mut i_episodes: IndexedVec<_> = mem::take(d_episodes)
		.into_iter()
		.map(|e| (e.episode as usize, e))
		.collect();

	// at first mark all episodes as removed
	for episode in &mut i_episodes {
		episode.change = Change::Remove;
	}

	// now we need to compare the episodes
	for episode in episodes {
		let Some(d_episode) = i_episodes.get_mut(episode.episode as usize)
		else {
			// insert a new one
			i_episodes.set(episode.episode as usize, episode.into());
			continue;
		};

		// now we know the episode already exists
		let mut changed = false;
		changes!(changed, d_episode, name = episode.name);

		d_episode.change.set_update(changed);
	}

	*d_episodes = i_episodes.into_iter().collect();
}

/// create a data entry from an Entry
impl From<Entry> for data::Entry {
	fn from(e: Entry) -> data::Entry {
		match e {
			Entry::Movie {
				name,
				year,
				duration,
				created_on,
				poster,
				background,
			} => data::Entry {
				id: UniqueId::new(),
				tmdb_id: None,
				name,
				original_name: None,
				description: None,
				poster,
				background,
				rating: None,
				data: data::EntryData::Movie(data::Movie {
					duration,
					year,
					progress: None,
				}),
				created_on: DateTime::from_std(created_on),
				updated_on: DateTime::from_std(created_on),
				genres: vec![],
				change: Change::Insert,
			},
			Entry::Series {
				name,
				seasons,
				poster,
				background,
			} => {
				let latest_updated_on = seasons
					.iter()
					.map(|s| s.episodes.iter().map(|e| e.created_on))
					.flatten()
					.max()
					.unwrap_or(SystemTime::UNIX_EPOCH);

				data::Entry {
					id: UniqueId::new(),
					tmdb_id: None,
					name,
					original_name: None,
					description: None,
					poster,
					background,
					rating: None,
					data: data::EntryData::Series(data::Series {
						seasons: seasons.into_iter().map(Into::into).collect(),
					}),
					// will be calculated later
					created_on: DateTime::from_std(latest_updated_on),
					updated_on: DateTime::from_std(latest_updated_on),
					genres: vec![],
					change: Change::Insert,
				}
			}
		}
	}
}

impl From<Season> for data::Season {
	fn from(s: Season) -> data::Season {
		data::Season {
			id: UniqueId::new(),
			season: s.season,
			name: s.name,
			original_name: None,
			episodes: s.episodes.into_iter().map(Into::into).collect(),
			change: Change::Insert,
			created_on: DateTime::now(),
		}
	}
}

impl From<Episode> for data::Episode {
	fn from(e: Episode) -> data::Episode {
		data::Episode {
			id: UniqueId::new(),
			episode: e.episode,
			name: e.name,
			original_name: None,
			year: None,
			created_on: DateTime::from_std(e.created_on),
			description: None,
			duration: None,
			change: Change::Insert,
			progress: None,
		}
	}
}
