//! Read files from the filesystem
//!
//! todo add multiple media versions
//! todo add background support

use super::util::IndexedVec;
use super::{Entry, EntryId, EntryKind, Episode, Season};
use crate::CinemaConf;

use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::str::FromStr;

use tokio::fs::{self, DirEntry};

use lazy_static::lazy_static;
use regex::Regex;

pub(super) async fn entries_from_fs(
	cfg: &CinemaConf,
) -> io::Result<HashMap<EntryId, Entry>> {
	let mut entries = HashMap::new();

	movies_from_fs(cfg, &mut entries).await?;
	series_from_fs(cfg, &mut entries).await?;

	Ok(entries)
}

fn io_error<E>(s: E) -> io::Error
where
	E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
	io::Error::new(io::ErrorKind::Other, s)
}

/// read movies from the filesystem
///
/// Expects the movies to be called `<name> <year>.mp4` and be in the movies folder
/// Expects the posters to be called `<name> <year>.jpg` and be in the movie_posters folder
async fn movies_from_fs(
	cfg: &CinemaConf,
	entries: &mut HashMap<EntryId, Entry>,
) -> io::Result<()> {
	// read movies
	let mut dir_reader = fs::read_dir(&cfg.movies_dir).await?;

	while let Some(dir_entry) = dir_reader.next_entry().await? {
		let metadata = dir_entry.metadata().await?;
		if !metadata.is_file() {
			continue;
		}

		let name = dir_entry.file_name().into_string().expect("non utf8 file");
		let Some(name) = name.strip_suffix(".mp4") else {
			continue;
		};

		let created = metadata.created()?;

		let poster_name = format!("{name}.jpg",);
		let poster_exists =
			fs::metadata(Path::new(&cfg.movie_posters_dir).join(&poster_name))
				.await
				.map(|m| m.is_file())
				.unwrap_or(false);

		let (name, year) = name
			.rsplit_once(' ')
			.ok_or_else(|| io_error("no year found"))?;

		let year: u16 = year.parse().map_err(|_| io_error("no year found"))?;

		let id = EntryId {
			kind: EntryKind::Movie,
			name: name.to_string(),
			year: Some(year),
		};

		entries.insert(
			id.clone(),
			Entry::Movie {
				name: name.to_string(),
				year,
				created_on: created,
				duration: 0,
				poster: poster_exists.then_some(poster_name),
				background: None,
			},
		);
	}

	Ok(())
}

fn entry_name(dir_entry: &DirEntry) -> String {
	dir_entry.file_name().into_string().expect("non utf8 file")
}

/// read series from the filesystem
///
/// Expects the series to be in the series folder
/// Uses the folder name as the series name
/// Expects `poster.jpg` files in the series folders
/// Expects `Season <number>` folders in the series folder
/// Expects `Episode <number> <name>.mp4` files in the season folders
///
/// series:
/// - Mr. Robot
/// - - poster.jpg
/// - - Season 1
/// - - - Episode 1 eps1.0_hellofriend.mp4
async fn series_from_fs(
	cfg: &CinemaConf,
	entries: &mut HashMap<EntryId, Entry>,
) -> io::Result<()> {
	// read movies
	let mut dir_reader = fs::read_dir(&cfg.series_dir).await?;
	while let Some(dir_entry) = dir_reader.next_entry().await? {
		let metadata = dir_entry.metadata().await?;
		if !metadata.is_dir() {
			continue;
		}

		let name = entry_name(&dir_entry);
		let path = dir_entry.path();

		let poster_name = "poster.jpg".to_string();
		let poster_exists = fs::metadata(path.join(&poster_name))
			.await
			.map(|m| m.is_file())
			.unwrap_or(false);

		let mut seasons = IndexedVec::new();

		let mut dir_reader = fs::read_dir(&path).await?;
		while let Some(dir_entry) = dir_reader.next_entry().await? {
			let metadata = dir_entry.metadata().await?;
			if !metadata.is_dir() {
				continue;
			}

			let season_name_str = entry_name(&dir_entry);
			let Ok(season_name) = season_name_str.parse::<SeasonName>() else {
				eprintln!("could not parse {season_name_str} in {name}");
				continue;
			};
			let season_path = dir_entry.path();

			// (EpisodeName, DateTime)
			let mut episodes = IndexedVec::new();

			let mut dir_reader = fs::read_dir(&season_path).await?;
			while let Some(dir_entry) = dir_reader.next_entry().await? {
				let metadata = dir_entry.metadata().await?;
				if !metadata.is_file() {
					continue;
				}

				let ep_name_str = entry_name(&dir_entry);
				let Some(ep_name) = ep_name_str.strip_suffix(".mp4") else {
					continue;
				};
				let Ok(ep_name) = ep_name.parse::<EpisodeName>() else {
					eprintln!(
						"could not parse {ep_name_str} in \
						{season_name_str} {name}"
					);
					continue;
				};

				let created = metadata.created()?;

				let already_exists = episodes
					.set(
						ep_name.number as usize - 1,
						Episode {
							name: ep_name.name,
							episode: ep_name.number,
							created_on: created,
						},
					)
					.is_some();
				if already_exists {
					return Err(io_error(format!(
						"episode {ep_name_str} with that number already \
							exists in {season_name_str} {name}"
					)));
				}
			}

			let episodes = episodes.into_iter().collect();
			let already_exists = seasons
				.set(
					season_name.number as usize - 1,
					Season {
						name: season_name.name,
						season: season_name.number,
						episodes,
					},
				)
				.is_some();
			if already_exists {
				return Err(io_error("season with that number already exists"));
			}
		}

		let id = EntryId {
			kind: EntryKind::Series,
			name: name.to_string(),
			year: None,
		};

		entries.insert(
			id.clone(),
			Entry::Series {
				name: name.to_string(),
				seasons: seasons.into_iter().collect(),
				poster: poster_exists.then_some(poster_name),
				background: None,
			},
		);
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
	number: u16,
	name: Option<String>,
}

impl FromStr for SeasonName {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, ()> {
		let caps = SEASON_REG.captures(s).ok_or(())?;
		let number: u16 = caps
			.get(1)
			.and_then(|c| c.as_str().parse().ok())
			.ok_or(())?;

		if number == 0 {
			return Err(());
		}

		let name = caps
			.get(2)
			.map(|c| c.as_str())
			.filter(|c| !c.is_empty())
			.map(|c| c.to_string());

		Ok(Self { number, name })
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EpisodeName {
	number: u16,
	name: String,
}

impl FromStr for EpisodeName {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, ()> {
		let caps = EPISODE_REG.captures(s).ok_or(())?;
		let number: u16 = caps
			.get(1)
			.and_then(|c| c.as_str().parse().ok())
			.ok_or(())?;

		if number == 0 {
			return Err(());
		}

		let name = caps.get(2).map(|c| c.as_str().to_string()).ok_or(())?;

		Ok(Self { number, name })
	}
}
