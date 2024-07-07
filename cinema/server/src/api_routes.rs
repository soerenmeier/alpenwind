use crate::api::{Entries, EntriesReq, ProgressMsg, ProgressReq};
use crate::data::{Entry, Progress};
use crate::error::{Error, Result};
use crate::fs::route::CinemaFsRoute;
use crate::{CinemaDb, Config};

use core_lib::users::Users;

use fire::header::RequestHeader;
use fire::{api, api_stream, FireBuilder};
use fire_api::stream::{StreamError, StreamServer, Streamer};

use postgres::time::DateTime;
use tracing::info;

#[api(EntriesReq)]
pub async fn entries(
	header: &RequestHeader,
	users: &Users,
	cinema: &CinemaDb,
) -> Result<Entries> {
	info!("entries req 1");
	let (_, user) = users.sess_user_from_req(header).await?;

	info!("entries req 2");

	Ok(Entries {
		list: cinema.all_by_user(&user.id).await?,
	})
}

#[api_stream(ProgressReq)]
pub async fn progress(
	req: ProgressReq,
	mut stream: Streamer<ProgressMsg>,
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
	server: &mut FireBuilder,
	stream_server: &mut StreamServer,
	cfg: &Config,
) {
	server.add_route(entries);
	stream_server.insert(progress);
	// server.add_route(CinemaFsRoute::new(&cfg.cinema));
}
