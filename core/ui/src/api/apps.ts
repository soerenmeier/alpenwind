import Api from 'fire/api/Api';

// @ts-ignore
const api = new Api(import.meta.env.SERVER_ADDR + 'api/apps/');

export class App {
	key: string;
	jsEntry?: string;
	cssEntry?: string;

	constructor(d: any) {
		Object.assign(this, d);
	}
}

export class AppsResp {
	apps: App[];

	constructor(d: any) {
		this.apps = d.apps.map((app: any) => new App(app));
	}
}

export async function apps() {
	const d = await api.request('GET', 'list');
	return new AppsResp(d).apps;
}
