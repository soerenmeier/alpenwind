pub mod db;
pub mod api;
pub mod api_routes;
pub use api_routes::sess_user_from_req;

pub use core_lib::users::{User, Rights, Token, Session, Timeout};

use tokio::time::{self, Duration};
use tokio::task::JoinHandle;

use fire::Data;


pub(crate) fn bg_task(data: Data) -> JoinHandle<()> {
	tokio::spawn(async move {
		let mut intv = time::interval(Duration::from_secs(2 * 60));
		let users = data.get::<db::Users>().unwrap();

		loop {
			intv.tick().await;

			users.sessions_cleanup();
		}
	})
}