mod data;
mod error;
mod api;
mod api_routes;
mod db;
mod favicons;

mod assets {
	include!(concat!(env!("OUT_DIR"), "/assets_routes.rs"));
}

use db::Passwords;

use core_lib::{init_fn, Core};
use core_lib::config::DbConf;
use core_lib::users::Users;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
	database: DbConf,
	pwvault: PwVaultConf
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PwVaultConf {
	#[serde(rename = "favicons-dir")]
	favicons_dir: String
}

init_fn!(init, "pwvault", assets::JS, assets::CSS);
async fn init(mut core: Core) {
	tracing_subscriber::fmt()
		.with_env_filter("error")
		.init();

	let cfg: Config = core.parse_config().expect("failed to read config1");

	// open database
	let db_cfg = &cfg.database;
	let db = postgres::Database::with_host(
		&db_cfg.host,
		&db_cfg.name,
		&db_cfg.user,
		&db_cfg.password
	).await;

	let users = Users::new(&db, core.sessions).await;
	let passwords = Passwords::new(&db).await;
	let favicons = favicons::Favicons::new();

	let mut server = core_lib::fire::build().await;

	server.add_data(users);
	server.add_data(passwords);
	server.add_data(favicons);
	server.add_data(cfg.pwvault.clone());

	assets::add_routes(&mut server);
	api_routes::add_routes(&mut server, &cfg);
	favicons::add_routes(&mut server);

	core_lib::fire::ignite(
		server,
		core.listener,
		 core.on_terminate.on_terminate()
	).await.unwrap()
}