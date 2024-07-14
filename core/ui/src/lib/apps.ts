import { Core } from 'core-lib';
import { apps as appsApi } from '../api/apps.js';
import Settings from '../settings/app.js';
import App from './App.js';

export let apps: App[] = [];

const addr = import.meta.env.SERVER_ADDR;

export class DynamicApp extends App {
	jsEntry: string;
	cssEntry: string;

	mod: any;
	info: { name: string } | null;

	constructor(a) {
		super(a.key);
		this.jsEntry = a.jsEntry;
		this.cssEntry = a.cssEntry;

		this.mod = null;
		this.info = null;
	}

	entry() {
		const name = this.jsEntry ?? 'main.js';
		return `${addr}assets/${this.key}/${name}`;
	}

	name() {
		return this.info?.name ?? '';
	}

	async prepare(cl: Core) {
		// @ts-ignore
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

	init(cl: Core) {
		this.info = this.mod.init(cl);
	}
}

/// we need to do this so vite can transform the files
async function importAppDev(key: string) {
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
export async function loadApps(cl: Core) {
	const list = await appsApi();
	apps = list.map(a => new DynamicApp(a));
	apps.push(new Settings());
	await Promise.all(apps.map(a => a.prepare(cl)));
	apps.forEach(a => a.init(cl));
}
