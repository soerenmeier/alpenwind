use crate::data::Password;
use crate::api::{AllReq, All, EditReq, DeleteReq};
use crate::{db, Config, Passwords};
use crate::error::Result;

use core_lib::users::Users;

use fire::{api, FireBuilder};
use fire::header::RequestHeader;

use postgres::UniqueId;
use postgres::time::DateTime;


#[api(AllReq)]
pub async fn all(
	header: &RequestHeader,
	users: &Users,
	passwords: &Passwords
) -> Result<All> {
	let (_, user) = users.sess_user_from_req(header).await?;

	let list = passwords.all_by_user(&user.id).await?;

	Ok(All { list })
}

#[api(EditReq)]
pub async fn edit(
	req: EditReq,
	header: &RequestHeader,
	users: &Users,
	passwords: &Passwords
) -> Result<Password> {
	let (_, user) = users.sess_user_from_req(header).await?;

	let create_new = req.id.is_none();

	let password = db::Password {
		id: req.id.unwrap_or_else(UniqueId::new),
		user_id: user.id,
		site: req.site,
		domain: req.domain,
		username: req.username,
		password: req.password,
		created_on: DateTime::now()
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
	req: DeleteReq,
	header: &RequestHeader,
	users: &Users,
	passwords: &Passwords
) -> Result<()> {
	let (_, user) = users.sess_user_from_req(header).await?;

	passwords.delete(&req.id, &user.id).await
		.map_err(Into::into)
}


pub(crate) fn add_routes(server: &mut FireBuilder, _cfg: &Config) {
	server.add_route(all);
	server.add_route(edit);
	server.add_route(delete);
}