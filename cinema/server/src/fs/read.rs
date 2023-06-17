use super::{EntryId, EntryKind, Entry, Season, UpdatedOnData};
use super::util::IndexedVec;
use crate::CinemaConf;

use std::io;
use std::collections::HashMap;
use std::str::FromStr;

use tokio::fs::{self, DirEntry};

use postgres::time::DateTime;

use lazy_static::lazy_static;
use regex::Regex;


pub(super) async fn entries_from_fs(
	cfg: &CinemaConf
) -> io::Result<HashMap<EntryId, (Entry, UpdatedOnData)>> {
	let mut entries = HashMap::new();

	movies_from_fs(cfg, &mut entries).await?;
	series_from_fs(cfg, &mut entries).await?;

	Ok(entries)
}

fn io_error<E>(s: E) -> io::Error
where E: Into<Box<dyn std::error::Error + Send + Sync>> {
	io::Error::new(io::ErrorKind::Other, s)
}

async fn movies_from_fs(
	cfg: &CinemaConf,
	entries: &mut HashMap<EntryId, (Entry, UpdatedOnData)>
) -> io::Result<()> {
	// read movies
	let mut dir_reader = fs::read_dir(&cfg.movies_dir).await?;

	while let Some(dir_entry) = dir_reader.next_entry().await? {
		let metadata = dir_entry.metadata().await?;
		if !metadata.is_file() {
			continue
		}

		let name = dir_entry.file_name().into_string().expect("non utf8 file");
		let Some(name) = name.strip_suffix(".mp4") else {
			continue
		};

		let created = DateTime::from_std(metadata.created()?);

		let poster_exists = fs::metadata(
			format!("{}/{name}.jpg", cfg.movie_posters_dir)
		).await.map(|m| m.is_file())
			.unwrap_or(false);

		if !poster_exists {
			eprintln!("poster for {name} not found");
			continue
		}

		let (name, year) = name.rsplit_once(' ')
			.ok_or_else(|| io_error("no year found"))?;

		let year: u32 = year.parse()
			.map_err(|_| io_error("no year found"))?;

		let id = EntryId {
			kind: EntryKind::Movie,
			name: name.to_string()
		};

		entries.insert(id.clone(), (
			Entry::Movie {
				name: name.to_string(),
				year
			},
			UpdatedOnData::Movie {
				updated_on: created
			}
		));
	}

	Ok(())
}

fn entry_name(dir_entry: &DirEntry) -> String {
	dir_entry.file_name().into_string().expect("non utf8 file")
}

async fn series_from_fs(
	cfg: &CinemaConf,
	entries: &mut HashMap<EntryId, (Entry, UpdatedOnData)>
) -> io::Result<()> {
	// read movies
	let mut dir_reader = fs::read_dir(&cfg.series_dir).await?;
	while let Some(dir_entry) = dir_reader.next_entry().await? {
		let metadata = dir_entry.metadata().await?;
		if !metadata.is_dir() {
			continue
		}

		let name = entry_name(&dir_entry);
		let path = dir_entry.path();

		let poster_exists = fs::metadata(path.join("poster.jpg")).await
			.map(|m| m.is_file())
			.unwrap_or(false);

		if !poster_exists {
			eprintln!("poster for {name} not found");
			continue
		}

		// (SeasonName, Vec<(String, DateTime)>)
		let mut seasons = IndexedVec::new();

		let mut dir_reader = fs::read_dir(&path).await?;
		while let Some(dir_entry) = dir_reader.next_entry().await? {
			let metadata = dir_entry.metadata().await?;
			if !metadata.is_dir() {
				continue
			}

			let season_name_str = entry_name(&dir_entry);
			let Ok(season_name) = season_name_str.parse::<SeasonName>() else {
				eprintln!("could not parse {season_name_str} in {name}");
				continue
			};
			let season_path = dir_entry.path();

			// (EpisodeName, DateTime)
			let mut episodes = IndexedVec::new();

			let mut dir_reader = fs::read_dir(&season_path).await?;
			while let Some(dir_entry) = dir_reader.next_entry().await? {
				let metadata = dir_entry.metadata().await?;
				if !metadata.is_file() {
					continue
				}

				let ep_name_str = entry_name(&dir_entry);
				let Some(ep_name) = ep_name_str.strip_suffix(".mp4") else {
					continue
				};
				let Ok(ep_name) = ep_name.parse::<EpisodeName>() else {
					eprintln!("could not parse {ep_name_str} in \
						{season_name_str} {name}");
					continue
				};

				let created = DateTime::from_std(metadata.created()?);

				let already_exists = episodes.set(
					ep_name.number as usize - 1,
					(ep_name, created)
				).is_some();
				if already_exists {
					return Err(
						io_error(format!(
							"episode {ep_name_str} with that number already \
							exists in {season_name_str} {name}"
						))
					)
				}
			}

			let episodes = episodes.into_contiguous_map(|(name, time)| {
				(name.name, time)
			});
			let already_exists = seasons.set(
				season_name.number as usize - 1,
				(season_name, episodes)
			).is_some();
			if already_exists {
				return Err(
					io_error("season with that number already exists")
				)
			}
		}

		let seasons = seasons.into_contiguous_map(|(name, eps)| {
			(name.name, eps)
		});
		let mut n_seasons = Vec::with_capacity(seasons.len());
		let mut update_on_seasons = Vec::with_capacity(seasons.len());

		for (name, eps) in seasons {
			let (eps, eps_update): (Vec<_>, Vec<_>) = eps.into_iter().unzip();

			n_seasons.push(Season { name, episodes: eps });
			update_on_seasons.push(eps_update);
		}

		let id = EntryId {
			kind: EntryKind::Series,
			name: name.to_string()
		};

		if n_seasons.is_empty() {
			continue
		}

		entries.insert(id.clone(), (
			Entry::Series {
				name: name.to_string(),
				seasons: n_seasons
			},
			UpdatedOnData::Series {
				seasons: update_on_seasons
			}
		));
	}

	Ok(())
}

lazy_static! {
	static ref SEASON_REG: Regex =
		Regex::new(r"^Season (\d\d)\s?(.*)").unwrap();
	static ref EPISODE_REG: Regex =
		Regex::new(r"^Episode (\d\d)\s(.+)").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SeasonName {
	number: u32,
	name: Option<String>
}

impl FromStr for SeasonName {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, ()> {
		let caps = SEASON_REG.captures(s).ok_or(())?;
		let number: u32 = caps.get(1)
			.and_then(|c| c.as_str().parse().ok())
			.ok_or(())?;

		if number == 0 {
			return Err(())
		}

		let name = caps.get(2)
			.map(|c| c.as_str())
			.filter(|c| !c.is_empty())
			.map(|c| c.to_string());

		Ok(Self { number, name })
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EpisodeName {
	number: u32,
	name: String
}

impl FromStr for EpisodeName {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, ()> {
		let caps = EPISODE_REG.captures(s).ok_or(())?;
		let number: u32 = caps.get(1)
			.and_then(|c| c.as_str().parse().ok())
			.ok_or(())?;

		if number == 0 {
			return Err(())
		}

		let name = caps.get(2)
			.map(|c| c.as_str().to_string())
			.ok_or(())?;

		Ok(Self { number, name })
	}
}