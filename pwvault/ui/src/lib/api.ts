import DateTime from 'chuchi-legacy/time/DateTime';
import { searchScore } from 'chuchi-utils/search';
import { Api } from 'chuchi/api';

// @ts-ignore
const addr: any = window.API_SERVER_ADDR;
const api = new Api(addr + 'api/pwvault/');

export function assets(url: string) {
	return `${addr}assets/pwvault/${url}`;
}

export class Password {
	id!: string;
	site!: string;
	domain!: string;
	username!: string;
	password!: string;
	createdOn!: DateTime;

	constructor(d: any) {
		Object.assign(this, d);
		this.createdOn = new DateTime(d.createdOn);
	}

	favicon() {
		if (!this.domain) return assets('favicon.png');
		return assets('favicons/' + this.domain);
	}

	match(val: string) {
		return searchScore(val, this.site) + searchScore(val, this.username);
	}
}

export class All {
	list: Password[];

	constructor(d: any) {
		this.list = d.list.map((p: any) => new Password(p));
	}
}

export async function all(token: string) {
	const d = await api.request('GET', 'passwords', null, {
		'auth-token': token,
	});
	return new All(d).list;
}

export class EditPassword {
	id: string | null;
	site: string;
	domain: string;
	username: string;
	password: string;

	constructor(p: any = null) {
		this.id = p?.id ?? null;
		this.site = p?.site ?? '';
		this.domain = p?.domain ?? '';
		this.username = p?.username ?? '';
		this.password = p?.password ?? '';
	}
}

// obj: EditPassword
export async function edit(obj: EditPassword, token: string) {
	const d = await api.request('POST', 'edit', obj, { 'auth-token': token });
	return new Password(d);
}

export async function delete_(id: string, token: string) {
	await api.request('DELETE', id + '/delete', null, { 'auth-token': token });
}
