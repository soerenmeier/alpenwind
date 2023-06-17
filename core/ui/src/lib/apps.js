import { apps as appsApi } from './../api/apps.js';
import Settings from './../settings/app.js';
import App from './app.js';

export let apps = [];

const addr = import.meta.env.SERVER_ADDR;

export class DynamicApp extends App {
	constructor(a) {
		super(a.key);
		this.jsEntry = a.jsEntry;
		this.cssEntry = a.cssEntry;

		this.mod = null;
	}

	entry() {
		const name = this.jsEntry ?? 'main.js';
		return `${addr}assets/${this.key}/${name}`;
	}

	name() {
		return this.info?.name ?? '';
	}

	async prepare() {
		if (import.meta.env.DEV) {
			this.mod = await importAppDev(this.key);
		} else {
			const link = document.createElement('link');
			link.rel = 'stylesheet';
			const name = this.cssEntry ?? 'style.css';
			link.href = `${addr}assets/${this.key}/${name}`;
			document.head.appendChild(link);
			this.mod = await import(/* @vite-ignore */ this.entry());
		}
	}

	init() {
		this.info = this.mod.init();
	}
}

/// we need to do this so vite can transform the files
async function importAppDev(key) {
	switch (key) {
		case 'cinema':
			return await import('../../../../cinema/ui/src/main.js');
		case 'pwvault':
			return await import('../../../../pwvault/ui/src/main.js');
		default:
			throw new Error('unknown dev path for app: ' + key);
	}
}

/// only needs to be called once
export async function loadApps() {
	const list = await appsApi();
	apps = list.map(a => new DynamicApp(a));
	apps.push(new Settings);
	await Promise.all(apps.map(a => a.prepare()));
	apps.forEach(a => a.init());
}