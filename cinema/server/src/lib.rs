mod api;
mod api_routes;
mod bg_task;
mod data;
mod db;
mod error;
mod fs;

mod assets {
	#[cfg(debug_assertions)]
	pub const JS: &str = "main.js";
	#[cfg(debug_assertions)]
	pub const CSS: &str = "style.css";

	include!(concat!(env!("OUT_DIR"), "/assets_routes.rs"));
}

use db::CinemaDb;

use core_lib::config::DbConf;
use core_lib::users::Users;
use core_lib::{init_fn, Core};

use chuchi::Resource;
use serde::{Deserialize, Serialize};

use chuchi::api::stream::StreamServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
	database: DbConf,
	cinema: CinemaConf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
struct CinemaConf {
	#[serde(rename = "movies-dir")]
	movies_dir: String,
	//
	#[serde(rename = "movie-posters-dir")]
	movie_posters_dir: String,
	//
	#[serde(rename = "series-dir")]
	series_dir: String,
	//
	#[serde(rename = "scaled-movies-posters")]
	scaled_movies_posters: String,
	//
	#[serde(rename = "scaled-series-posters")]
	scaled_series_posters: String,
	//
	#[serde(rename = "allow-deletes", default)]
	allow_deletes: bool,
}

init_fn!(init, "cinema", assets::JS, assets::CSS);
async fn init(core: Core) {
	tracing_subscriber::fmt()
		.with_env_filter("cinema_server=info,chuchi=info,warn")
		.init();

	let cfg: Config = core.parse_config().expect("failed to read config");

	// open database
	let db_cfg = &cfg.database;
	let db = chuchi_postgres::Database::with_host(
		&db_cfg.host,
		&db_cfg.name,
		&db_cfg.user,
		&db_cfg.password,
	)
	.await
	.unwrap();

	let users = Users::new(&db, core.sessions).await;
	let cinema = CinemaDb::new(&db).await;

	let mut server = core_lib::chuchi::build().await;
	let mut stream_server = StreamServer::new("/api/cinema/stream");

	server.add_resource(users);
	server.add_resource(cinema);
	server.add_resource(cfg.cinema.clone());

	assets::add_routes(&mut server);
	api_routes::add_routes(&mut server, &mut stream_server);
	fs::route::add_routes(&mut server);

	server.add_raw_route(stream_server);

	let on_terminate = core.on_terminate.clone();
	tokio::try_join! {
		bg_task::bg_task(
			server.resources().clone(),
			cfg.cinema,
			core.on_terminate.clone()
		),
		tokio::spawn(async move {
			core_lib::chuchi::ignite(
				server,
				core.listener,
				on_terminate
			).await.unwrap()
		})
	}
	.unwrap();
}
