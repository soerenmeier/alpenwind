import { match } from 'fire/util.js';
import Api from 'fire/api/api.js';
import Data from 'fire/data/data.js';
import * as core from 'core-lib';

const addr = core.api.serverAddr;
const api = new Api(addr + 'api/pwvault/');

export function assets(url) {
	return `${addr}assets/pwvault/${url}`;
}

export class Password extends Data {
	constructor(d) {
		super({
			id: 'uid',
			site: 'str',
			domain: 'str',
			username: 'str',
			password: 'str',
			createdOn: 'datetime'
		}, d);
	}

	favicon() {
		return assets('favicons/' + this.domain);
	}

	match(val) {
		return match(val, this.site) + match(val, this.username);
	}
}

export class All extends Data {
	constructor(d) {
		super({
			list: [Password]
		}, d);
	}
}

export async function all(token) {
	const d = await api.request('POST', 'passwords', null,
		{ 'auth-token': token });
	return (new All(d)).list;
}

export class EditPassword {
	constructor(p = null) {
		this.id = p?.id ?? null;
		this.site = p?.site ?? '';
		this.domain = p?.domain ?? '';
		this.username = p?.username ?? '';
		this.password = p?.password ?? '';
	}
}

// obj: EditPassword
export async function edit(obj, token) {
	const d = await api.request('POST', 'edit', obj, { 'auth-token': token });
	return new Password(d);
}

export async function delete_(id, token) {
	await api.request('POST', 'delete', { id }, { 'auth-token': token });
}