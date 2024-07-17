use std::marker::PhantomData;

use chuchi::{
	extractor::Extractor, extractor_extract, extractor_prepare,
	extractor_validate, header::RequestHeader,
};

use crate::users::Users;

use super::{Error, Rights, Session, Token, User};

pub struct CheckedUser<RightsCheck = RightsRoot, AuthToken = NormalToken> {
	pub session: Session,
	pub user: User,
	rights_check: PhantomData<RightsCheck>,
	auth_token: PhantomData<AuthToken>,
}

impl<'a, R, RC, AT> Extractor<'a, R> for CheckedUser<RC, AT>
where
	RC: RightsCheck,
	AT: AuthToken,
{
	type Error = Error;
	type Prepared = Self;

	extractor_validate!(|validate| {
		assert!(
			validate.resources.exists::<Users>(),
			"Users resource not found"
		);
	});

	extractor_prepare!(|prepare| {
		let users = prepare.resources.get::<Users>().unwrap();

		let token = AT::get_token(prepare.header)?;

		let (session, user) = match token {
			TokenKind::Normal(token) => {
				users.sess_user_from_token(&token).await?
			}
			TokenKind::Data(token) => {
				users.sess_user_from_data_token(&token).await?
			}
		};

		Ok(Self {
			session,
			user,
			rights_check: PhantomData,
			auth_token: PhantomData,
		})
	});

	extractor_extract!(|extract| { Ok(extract.prepared) });
}

pub trait RightsCheck {
	fn check(rights: &Rights) -> bool;
}

pub struct RightsAny;

impl RightsCheck for RightsAny {
	fn check(_rights: &Rights) -> bool {
		true
	}
}

pub struct RightsRoot;

impl RightsCheck for RightsRoot {
	fn check(rights: &Rights) -> bool {
		rights.root
	}
}

pub enum TokenKind {
	Normal(Token),
	Data(Token),
}

pub trait AuthToken {
	fn get_token(header: &RequestHeader) -> Result<TokenKind, Error>;
}

pub struct NormalToken;

impl AuthToken for NormalToken {
	fn get_token(header: &RequestHeader) -> Result<TokenKind, Error> {
		super::get_token(header)
			.ok_or(Error::MissingAuthToken)
			.map(TokenKind::Normal)
	}
}

pub struct DataToken;

impl AuthToken for DataToken {
	fn get_token(header: &RequestHeader) -> Result<TokenKind, Error> {
		super::get_token_from_cookie(header)
			.ok_or(Error::MissingAuthToken)
			.map(TokenKind::Data)
	}
}
