use crate::api::data::{Entry, Progress};
use crate::api::{Entries, EntriesReq, ProgressId, ProgressMsg, ProgressReq};
use crate::data;
use crate::db::{self, CinemaDb};
use crate::error::{Error, Result};

use chuchi_postgres::connection::ConnectionOwned;
use chuchi_postgres::Database;
use core_lib::users::{CheckedUser, Users};

use chuchi::api::stream::{StreamError, StreamServer, Streamer};
use chuchi::{api, api_stream, Chuchi, Res};

use chuchi_postgres::time::DateTime;

#[api(EntriesReq)]
pub async fn entries(
	conn: ConnectionOwned,
	cinema: &CinemaDb,
	sess: CheckedUser,
) -> Result<Entries> {
	let cinema = cinema.with_conn(conn.connection());

	Ok(Entries {
		list: cinema
			.all_by_user(&sess.user.id)
			.await?
			.into_iter()
			.map(|(_, e)| e.into())
			.collect(),
	})
}

#[api_stream(ProgressReq)]
pub async fn progress(
	req: ProgressReq,
	mut stream: Streamer<ProgressMsg>,
	database: Res<'_, Database>,
	cinema: &CinemaDb,
	users: &Users,
) -> Result<()> {
	let (_, user) = users.sess_user_from_token(&req.token).await?;

	loop {
		let msg = match stream.recv().await {
			Ok(m) => m,
			Err(StreamError::Closed) => return Ok(()),
			Err(StreamError::Json(e)) => {
				return Err(Error::Request(e.to_string()))
			}
		};

		let mut conn = database
			.get()
			.await
			.map_err(|e| Error::Internal(e.to_string()))?;
		let cinema_r = cinema.with_conn(conn.connection());

		// check if the associated id exists and the create a progress
		let mut movie_id = None;
		let mut episode_id = None;
		let exists = match msg.id {
			ProgressId::Movie(id) => {
				movie_id = Some(id);

				cinema_r.movie_exists(&id).await?
			}
			ProgressId::Episode(id) => {
				episode_id = Some(id);

				cinema_r.episode_exists(&id).await?
			}
		};

		if !exists {
			return Err(Error::NotFound);
		}

		drop(cinema_r);
		// open a transaction
		let trans = conn.transaction().await?;
		let cinema = cinema.with_conn(trans.connection());

		let mut prog = cinema
			.progress_by_id_user(&movie_id, &episode_id, &user.id)
			.await?
			.unwrap_or_else(|| db::Progress {
				entry_id: movie_id,
				episode_id,
				user_id: user.id,
				progress: 0f32,
				created_on: DateTime::now(),
				updated_on: DateTime::now(),
				last_watch: None,
			});

		// todo do we wan't it like this?
		if msg.percent > 0.9999 {
			prog.last_watch = Some(DateTime::now());
		}
		prog.progress = msg.percent;
		prog.updated_on = DateTime::now();

		cinema.update_progress(prog).await?;

		trans.commit().await?;
	}
}

//

pub(crate) fn add_routes(
	server: &mut Chuchi,
	stream_server: &mut StreamServer,
) {
	server.add_route(entries);
	stream_server.insert(progress);
}
