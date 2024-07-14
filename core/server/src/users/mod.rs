pub mod api;
pub mod api_routes;
pub mod db;

pub use core_lib::users::{Rights, Session, Timeout, Token, User};

use chuchi::resources::Resources;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

pub(crate) fn bg_task(data: Resources) -> JoinHandle<()> {
	tokio::spawn(async move {
		let mut intv = time::interval(Duration::from_secs(2 * 60));
		let users = data.get::<db::Users>().unwrap();

		loop {
			intv.tick().await;

			users.sessions_cleanup();
		}
	})
}
