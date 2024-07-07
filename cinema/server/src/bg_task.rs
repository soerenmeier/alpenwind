use crate::db::CinemaDb;
use crate::error::{Error, Result};
use crate::fs::{changes_from_fs, EntryChange};
use crate::CinemaConf;

use fire::resources::Resources;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

use core_lib::server::OnTerminate;

#[cfg(debug_assertions)]
const REFRESH_EVERY: Duration = Duration::from_secs(1 * 60);
#[cfg(not(debug_assertions))]
const REFRESH_EVERY: Duration = Duration::from_secs(5 * 60);

pub(crate) fn bg_task(
	data: Resources,
	cfg: CinemaConf,
	mut on_terminate: OnTerminate,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let mut intv = time::interval(REFRESH_EVERY);
		let cinema: &CinemaDb = data.get().unwrap();

		let terminate = on_terminate.on_terminate();
		tokio::pin!(terminate);
		loop {
			tokio::select! {
				_ = intv.tick() => {},
				_ = &mut terminate => return
			}

			if let Err(e) = task_tick(cinema, &cfg).await {
				eprintln!("failed to update cinema {e:?}");
			}
		}
	})
}

async fn task_tick(cinema: &CinemaDb, cfg: &CinemaConf) -> Result<()> {
	let entries = cinema.all().await?;

	let changes = changes_from_fs(&entries, &cfg)
		.await
		.map_err(|e| Error::Internal(e.to_string()))?;

	// todo this is not optimal but big insert should not happen often
	for change in changes {
		match change {
			EntryChange::Insert(entry) => {
				eprintln!("cinema insert {entry:?}");
				cinema.insert_data(&entry).await?;
			}
			EntryChange::Update(entry) => {
				eprintln!("cinema update {entry:?}");
				cinema.update_data(&entry).await?;
			}
			EntryChange::Remove(id) => {
				eprintln!("cinema remove {id:?}");
				if cfg.allow_deletes {
					cinema.delete_by_id(&id).await?;
				}
			}
		}
	}

	Ok(())
}
