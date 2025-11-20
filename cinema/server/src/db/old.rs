use chuchi::Resource;
use chuchi_postgres::json::Json;
use chuchi_postgres::table::table::TableWithConn;
use chuchi_postgres::table::Table;
use chuchi_postgres::time::DateTime;
use chuchi_postgres::{filter, Connection, FromRow, ToRow};
use chuchi_postgres::{Database, Result, UniqueId};

use serde::{Deserialize, Serialize};

/// Todo redo without TableTempl
#[derive(Debug, FromRow, ToRow)]
pub struct Entry {
	pub id: UniqueId,
	pub name: String,
	pub updated_on: DateTime,
	pub data: Json<EntryData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryData {
	Movie { year: u32 },
	Series { seasons: Vec<Season> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
	pub name: Option<String>,
	pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
	pub name: String,
	pub updated_on: DateTime,
}

#[derive(Debug, FromRow, ToRow)]
pub struct EntryProgress {
	pub entry_id: UniqueId,
	pub user_id: UniqueId,
	pub updated_on: DateTime,
	pub data: Json<EntryProgressData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryProgressData {
	Movie {
		progress: Progress,
	},
	Series {
		seasons: Vec<Vec<Option<EpisodeProgress>>>,
	},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeProgress {
	pub progress: Progress,
	pub updated_on: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
	pub percent: f32,
	pub position: f32,
}

#[derive(Resource)]
pub struct CinemaDb {
	table: Table,
	table_progress: Table,
}

impl CinemaDb {
	pub async fn new(db: &Database) -> Self {
		let this = Self {
			table: Table::new("cinema_old"),
			table_progress: Table::new("cinema_progress_old"),
		};

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
	pub table: TableWithConn<'a>,
	pub table_progress: TableWithConn<'a>,
}

impl CinemaDbWithConn<'_> {
	pub async fn all_entries(&self) -> Result<Vec<Entry>> {
		self.table.select(filter!()).await
	}

	pub async fn all_progress(&self) -> Result<Vec<EntryProgress>> {
		self.table_progress.select(filter!()).await
	}
}
