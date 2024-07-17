mod api;
mod api_routes;
mod data;
mod db;
mod error;
mod favicons;

mod assets {
	#[cfg(debug_assertions)]
	pub const JS: &str = "main.js";
	#[cfg(debug_assertions)]
	pub const CSS: &str = "style.css";

	include!(concat!(env!("OUT_DIR"), "/assets_routes.rs"));
}

use db::Passwords;

use core_lib::config::DbConf;
use core_lib::users::Users;
use core_lib::{init_fn, Core};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
	database: DbConf,
	pwvault: PwVaultConf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PwVaultConf {
	#[serde(rename = "favicons-dir")]
	favicons_dir: String,
}

init_fn!(init, "pwvault", assets::JS, assets::CSS);
async fn init(core: Core) {
	tracing_subscriber::fmt()
		.with_env_filter("pwvault_server=info,chuchi=info,warn")
		.init();

	let cfg: Config = core.parse_config().expect("failed to read config1");

	// open database
	let db_cfg = &cfg.database;
	let db = chuchi_postgres::Database::with_host(
		&db_cfg.host,
		&db_cfg.name,
		&db_cfg.user,
		&db_cfg.password,
	)
	.await
	.expect("failed to connect to database");

	let users = Users::new(&db, core.sessions).await;
	let passwords = Passwords::new(&db).await;
	let favicons = favicons::Favicons::new();

	let mut server = core_lib::chuchi::build().await;

	server.add_resource(users);
	server.add_resource(passwords);
	server.add_resource(favicons);
	server.add_resource(cfg.pwvault.clone());

	assets::add_routes(&mut server);
	api_routes::add_routes(&mut server, &cfg);
	favicons::add_routes(&mut server);

	core_lib::chuchi::ignite(server, core.listener, core.on_terminate)
		.await
		.unwrap()
}
