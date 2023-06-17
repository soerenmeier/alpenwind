use super::{User, Session, Timeout};
use super::api::{LoginReq, Login, LoginByTokenReq, RenewReq, LogoutReq, SaveReq};
use super::db::Users;
use crate::api::{Result, Error};

use std::time::Duration;

use fire::FireBuilder;
use fire::header::RequestHeader;

use fire_api::api;
use fire_api::util::ResponseHeaders;

use core_lib::users::{get_token, get_token_from_cookie};

// 1/2 year
const TIMEOUT_DURATION: Duration = Duration::from_secs(365 * 24 * 60 * 60 * 60);


pub async fn sess_user_from_req(
	header: &RequestHeader,
	users: &Users
) -> Result<(Session, User)> {
	let token = get_token(header)
		.ok_or(Error::MissingAuthToken)?;

	users.by_sess_token(&token).await?
		.ok_or(Error::InvalidAuthToken)
}

#[allow(dead_code)]
pub async fn sess_user_from_cookie(
	header: &RequestHeader,
	users: &Users
) -> Result<(Session, User)> {
	let token = get_token_from_cookie(header)
		.ok_or(Error::MissingDataToken)?;

	users.by_data_token(&token).await?
		.ok_or(Error::InvalidDataToken)
}

fn set_cookie(headers: &mut ResponseHeaders, sess: Option<&Session>) {
	let setts = if crate::Args::enable_cors() {
		"Path=/; HttpOnly; SameSite=None; Secure"
	} else {
		"Path=/; HttpOnly; SameSite=Strict; Secure"
	};

	if let Some(sess) = sess {
		headers.insert(
			"set-cookie",
			format!("data-token={}; {setts}", sess.data_token)
		);
	} else {
		headers.insert("set-cookie", format!("data-token=; {setts}"));
	}
}

#[api(LoginReq)]
async fn login(
	req: LoginReq,
	users: &Users,
	resp_header: &mut ResponseHeaders
) -> Result<Login> {
	let user = users.login(&req.username, &req.password).await?
		.ok_or(Error::LoginIncorrect)?;

	// create Session
	let session = users.session_insert(user.id, Timeout::new(TIMEOUT_DURATION));

	set_cookie(resp_header, Some(&session));

	Ok(Login { user, session })
}

#[api(LoginByTokenReq)]
async fn login_by_token(
	header: &RequestHeader,
	users: &Users,
	resp_header: &mut ResponseHeaders
) -> Result<Login> {
	let (session, user) = sess_user_from_req(header, users).await?;

	set_cookie(resp_header, Some(&session));

	Ok(Login { user, session })
}

#[api(RenewReq)]
async fn renew(
	header: &RequestHeader,
	users: &Users,
	resp_header: &mut ResponseHeaders
) -> Result<Login> {
	let (session, user) = sess_user_from_req(header, users).await?;

	users.session_remove(&session.token);

	// create Session
	let session = users.session_insert(user.id, Timeout::new(TIMEOUT_DURATION));

	set_cookie(resp_header, Some(&session));

	Ok(Login { user, session })
}

#[api(LogoutReq)]
async fn logout(
	header: &RequestHeader,
	users: &Users,
	resp_header: &mut ResponseHeaders
) -> Result<()> {
	let (session, _) = sess_user_from_req(header, users).await?;

	users.session_remove(&session.token);

	// set cookies
	set_cookie(resp_header, None);

	Ok(())
}

#[api(SaveReq)]
async fn save(
	req: SaveReq,
	header: &RequestHeader,
	users: &Users
) -> Result<User> {
	let (_, mut user) = sess_user_from_req(header, users).await?;

	user.name = req.name;
	users.update(&user.id, &user.name, req.password.as_deref()).await?;

	Ok(user)
}


pub fn add_routes(server: &mut FireBuilder) {
	server.add_route(login);
	server.add_route(login_by_token);
	server.add_route(renew);
	server.add_route(logout);
	server.add_route(save);
}