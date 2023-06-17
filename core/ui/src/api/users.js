import Api from 'fire/api/api.js';
import Data from 'fire/data/data.js';
import { user as userMod } from 'core-lib';
const { User, Session } = userMod;

const api = new Api(import.meta.env.SERVER_ADDR + 'api/users/');

export class Login extends Data {
	constructor(d) {
		super({
			user: User,
			session: Session
		}, d);
	}
}

export async function login(username, password) {
	username = username.toLowerCase();
	const d = await api.request('POST', 'login', { username, password },
		{ credentials: 'include' });
	return new Login(d);
}

export async function loginByToken(token) {
	const d = await api.request('POST', 'loginbytoken', null,
		{ 'auth-token': token }, { credentials: 'include' });
	return new Login(d);
}

export async function renew(token) {
	const d = await api.request('POST', 'renew', null,
		{ 'auth-token': token }, { credentials: 'include' });
	return new Login(d);
}

export async function logout(token) {
	await api.request('POST', 'logout', null, { 'auth-token': token },
		{ credentials: 'include' });
}

export async function save(name, password, token) {
	if (!password)
		password = null;
	const d = await api.request('POST', 'save', { name, password },
		{ 'auth-token': token });
	return new User(d);
}