import Data from 'fire/data/data.js';
import Api from 'fire/api/api.js';

const api = new Api(import.meta.env.SERVER_ADDR + 'api/apps/');

export class App extends Data {
	constructor(d) {
		super({
			key: 'str',
			jsEntry: 'optstr',
			cssEntry: 'optstr'
		}, d);
	}
}

export class AppsResp extends Data {
	constructor(d) {
		super({
			apps: [App]
		}, d);
	}
}

export async function apps() {
	const d = await api.request('GET', 'list');
	return (new AppsResp(d)).apps;
}