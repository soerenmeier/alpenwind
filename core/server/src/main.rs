mod api;
mod apps;
mod cors;
#[cfg(not(debug_assertions))]
mod index;
mod tempfile;
mod users;

mod assets {
	include!(concat!(env!("OUT_DIR"), "/assets_routes.rs"));
}

use users::db::Users;
use users::Rights;

use tokio::fs;

use core_lib::config::DbConf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
	#[clap(subcommand)]
	subcmd: Option<SubCommand>,
	#[clap(long)]
	enable_cors: bool,
	#[clap(long, default_value = "./config.toml")]
	config: String,
}

#[derive(Debug, Parser)]
enum SubCommand {
	CreateUser(CreateUser),
}

#[derive(Debug, Parser)]
struct CreateUser {
	username: String,
	name: String,
	password: String,
	#[clap(long)]
	root: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
	#[serde(rename = "listen-on")]
	listen_on: String,
	database: DbConf,
	apps: apps::AppsConf,
}

struct ConfigString(String);

#[tokio::main]
async fn main() {
	let mut args = Args::parse();
	unsafe { args.init() };

	tracing_subscriber::fmt()
		.with_env_filter("core-server=info,chuchi=info,error")
		.init();

	let cfg_string = fs::read_to_string(args.config)
		.await
		.expect("failed to read config.toml");
	let cfg: Config =
		toml::from_str(&cfg_string).expect("failed to read config.toml");
	let cfg_string = ConfigString(cfg_string);

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

	let users = Users::new(&db).await;

	match args.subcmd {
		Some(SubCommand::CreateUser(create_user)) => {
			let rights = Rights {
				root: create_user.root,
			};

			let user = users
				.insert(
					create_user.username,
					create_user.name,
					create_user.password,
					rights,
				)
				.await
				.unwrap();
			println!("created user {user:?}");
			return;
		}
		None => {}
	}

	let mut server = chuchi::build(&cfg.listen_on).await.unwrap();

	server.add_resource(users);
	server.add_resource(apps::Apps::new());
	server.add_resource(cfg_string);
	assets::add_routes(&mut server);
	users::api_routes::add_routes(&mut server);
	server.add_raw_route(apps::route::AppsApiRoute);
	server.add_raw_route(apps::route::AppsAssetsRoute);
	apps::api_routes::add_routes(&mut server);
	#[cfg(not(debug_assertions))]
	index::add_routes(&mut server);
	if Args::enable_cors() {
		cors::add_routes(&mut server);
	}

	let data = server.resources().clone();

	tokio::try_join!(
		users::bg_task(data.clone()),
		apps::bg_task(&cfg.apps, data.clone()),
		tokio::spawn(async move {
			server.run().await.unwrap();
		})
	)
	.unwrap();
}

static mut ENABLE_CORS: bool = false;
impl Args {
	// only allowed to be called before others have access to cors
	unsafe fn init(&mut self) {
		self.enable_cors = cfg!(debug_assertions) || self.enable_cors;
		ENABLE_CORS = self.enable_cors;
	}

	fn enable_cors() -> bool {
		unsafe { ENABLE_CORS }
	}
}
