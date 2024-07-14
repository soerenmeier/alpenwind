mod read;
pub(super) mod route;
mod util;

use super::data;
use crate::CinemaConf;

use std::collections::HashMap;
use std::io;

use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EntryId {
	kind: EntryKind,
	name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum EntryKind {
	Movie,
	Series,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
	Movie { name: String, year: u32 },
	Series { name: String, seasons: Vec<Season> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Season {
	name: Option<String>,
	episodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum UpdatedOnData {
	Movie { updated_on: DateTime },
	Series { seasons: Vec<Vec<DateTime>> },
}

#[derive(Debug, Clone)]
pub(super) enum EntryChange {
	Insert(data::Entry),
	Update(data::Entry),
	Remove(UniqueId),
}

pub(super) async fn changes_from_fs(
	data_entries: &[data::Entry],
	cfg: &CinemaConf,
) -> io::Result<Vec<EntryChange>> {
	let mut entries = convert_entries(data_entries);

	let fs_entries = read::entries_from_fs(cfg).await?;

	let mut changes = vec![];

	for (id, (entry, updated_on)) in fs_entries {
		match entries.remove(&id) {
			Some((uid, db_entry)) => {
				if entry == db_entry {
					// the entries are the same don't do any change
					continue;
				}

				let data_entry = entry.into_data(updated_on, uid);
				changes.push(EntryChange::Update(data_entry));
			}
			None => {
				// entry does not exist create a new one
				let data_entry = entry.into_data(updated_on, UniqueId::new());
				changes.push(EntryChange::Insert(data_entry));
			}
		}
	}

	// now check if some data should be deleted
	for (_, (uid, _)) in entries {
		changes.push(EntryChange::Remove(uid));
	}

	Ok(changes)
}

fn convert_entries(
	entries: &[data::Entry],
) -> HashMap<EntryId, (UniqueId, Entry)> {
	let mut map = HashMap::with_capacity(entries.len());

	for entry in entries {
		match entry {
			data::Entry::Movie(m) => {
				let id = EntryId {
					kind: EntryKind::Movie,
					name: m.name.clone(),
				};

				let entry = Entry::Movie {
					name: m.name.clone(),
					year: m.year,
				};

				map.insert(id, (m.id, entry));
			}
			data::Entry::Series(s) => {
				let id = EntryId {
					kind: EntryKind::Series,
					name: s.name.clone(),
				};

				let entry = Entry::Series {
					name: s.name.clone(),
					seasons: s
						.seasons
						.iter()
						.map(|s| Season {
							name: s.name.clone(),
							episodes: s
								.episodes
								.iter()
								.map(|e| e.name.clone())
								.collect(),
						})
						.collect(),
				};

				map.insert(id, (s.id, entry));
			}
		}
	}

	map
}

impl Entry {
	fn into_data(self, updated_on: UpdatedOnData, id: UniqueId) -> data::Entry {
		match (self, updated_on) {
			(
				Entry::Movie { name, year },
				UpdatedOnData::Movie { updated_on },
			) => data::Entry::Movie(data::Movie {
				id,
				name,
				year,
				updated_on,
				progress: None,
			}),
			(
				Entry::Series { name, seasons },
				UpdatedOnData::Series {
					seasons: updated_on_seasons,
				},
			) => {
				assert_eq!(seasons.len(), updated_on_seasons.len());

				data::Entry::Series(data::Series {
					id,
					name,
					seasons: seasons
						.into_iter()
						.zip(updated_on_seasons.into_iter())
						.map(|(s, u)| {
							assert_eq!(s.episodes.len(), u.len());

							data::Season {
								name: s.name,
								episodes: s
									.episodes
									.into_iter()
									.zip(u.into_iter())
									.map(|(e, u)| data::Episode {
										name: e,
										updated_on: u,
										progress: None,
									})
									.collect(),
							}
						})
						.collect(),
				})
			}
			_ => unreachable!(),
		}
	}
}
