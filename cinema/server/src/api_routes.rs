use crate::api::{Entries, EntriesReq, ProgressMsg, ProgressReq};
use crate::data::{Entry, Progress};
use crate::error::{Error, Result};
use crate::CinemaDb;

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
		list: cinema.all_by_user(&sess.user.id).await?,
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
		let conn = database
			.get()
			.await
			.map_err(|e| Error::Internal(e.to_string()))?;
		let cinema = cinema.with_conn(conn.connection());

		let msg = match stream.recv().await {
			Ok(m) => m,
			Err(StreamError::Closed) => return Ok(()),
			Err(StreamError::Json(e)) => {
				return Err(Error::Request(e.to_string()))
			}
		};

		match msg {
			ProgressMsg::Movie(m) => {
				// get the data
				let mut entry =
					cinema.by_id_and_user(&m.id, &user.id).await?.ok_or_else(
						|| Error::Request("movie not found".into()),
					)?;

				let movie = match &mut entry {
					Entry::Movie(m) => m,
					_ => return Err(Error::Request("not a movie".into())),
				};

				movie.progress = Some(Progress {
					percent: m.percent,
					position: m.position,
					updated_on: DateTime::now(),
				});

				cinema.update_progress(&entry, &user.id).await?;
			}
			ProgressMsg::Series(s) => {
				// get the data
				let mut entry =
					cinema.by_id_and_user(&s.id, &user.id).await?.ok_or_else(
						|| Error::Request("series not found".into()),
					)?;

				let series = match &mut entry {
					Entry::Series(s) => s,
					_ => return Err(Error::Request("not a series".into())),
				};

				let episode = series
					.seasons
					.get_mut(s.season as usize)
					.and_then(|season| {
						season.episodes.get_mut(s.episode as usize)
					})
					.ok_or_else(|| {
						Error::Request("episode not found".into())
					})?;

				episode.progress = Some(Progress {
					percent: s.percent,
					position: s.position,
					updated_on: DateTime::now(),
				});

				cinema.update_progress(&entry, &user.id).await?;
			}
		}
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
