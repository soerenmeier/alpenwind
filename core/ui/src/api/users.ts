import { Api } from 'chuchi/api';
import { user as userMod } from 'core-lib';
const { User, Session } = userMod;

const api = new Api(import.meta.env.SERVER_ADDR + 'api/users/');

export class Login {
	user: userMod.User;
	session: userMod.Session;

	constructor(d: any) {
		this.user = new User(d.user);
		this.session = new Session(d.session);
	}
}

export async function login(
	username: string,
	password: string,
): Promise<Login> {
	username = username.toLowerCase();
	const d = await api.request(
		'POST',
		'login',
		{ username, password },
		{ credentials: 'include' },
	);

	return new Login(d);
}

export async function loginByToken(token: string) {
	const d = await api.request(
		'POST',
		'loginbytoken',
		null,
		{ 'auth-token': token },
		{ credentials: 'include' },
	);

	return new Login(d);
}

export async function renew(token: string) {
	const d = await api.request(
		'POST',
		'renew',
		null,
		{ 'auth-token': token },
		{ credentials: 'include' },
	);

	return new Login(d);
}

export async function logout(token: string) {
	await api.request(
		'POST',
		'logout',
		null,
		{ 'auth-token': token },
		{ credentials: 'include' },
	);
}

export async function save(
	name: string,
	password: string | null,
	token: string,
) {
	if (!password) password = null;
	const d = await api.request(
		'POST',
		'save',
		{ name, password },
		{ 'auth-token': token },
	);

	return new User(d);
}
