use crate::api::{All, AllReq, DeleteReq, EditReq};
use crate::data::Password;
use crate::error::Result;
use crate::{db, Config, Passwords};

use chuchi::extractor::PathParam;
use chuchi::routes::PathParams;
use core_lib::users::{CheckedUser, Users};

use chuchi::header::RequestHeader;
use chuchi::{api, Chuchi};

use chuchi_postgres::time::DateTime;
use chuchi_postgres::UniqueId;

#[api(AllReq)]
pub async fn all(sess: CheckedUser, passwords: &Passwords) -> Result<All> {
	let list = passwords.all_by_user(&sess.user.id).await?;

	Ok(All { list })
}

#[api(EditReq)]
pub async fn edit(
	req: EditReq,
	sess: CheckedUser,
	passwords: &Passwords,
) -> Result<Password> {
	let create_new = req.id.is_none();

	let password = db::Password {
		id: req.id.unwrap_or_else(UniqueId::new),
		user_id: sess.user.id,
		site: req.site,
		domain: req.domain,
		username: req.username,
		password: req.password,
		created_on: DateTime::now(),
	};

	if create_new {
		passwords.insert(&password).await?;
	} else {
		passwords.update(&password).await?;
	}

	Ok(password.into())
}

#[api(DeleteReq)]
pub async fn delete(
	id: PathParam<UniqueId>,
	sess: CheckedUser,
	passwords: &Passwords,
) -> Result<()> {
	passwords
		.delete(&id, &sess.user.id)
		.await
		.map_err(Into::into)
}

pub(crate) fn add_routes(server: &mut Chuchi, _cfg: &Config) {
	server.add_route(all);
	server.add_route(edit);
	server.add_route(delete);
}
