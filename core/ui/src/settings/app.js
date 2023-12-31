import App from './../lib/app.js';
import Page from './page.svelte';
import * as core from 'core-lib';
import settingsIcon from './../../assets/settings/icon.png';
const { StaticRoute, SvelteComponent } = core.router;


export default class Settings extends App {
	constructor() {
		super('settings');
	}

	icon() {
		return settingsIcon;
	}

	name() {
		return 'Istellige';
	}

	init(cl) {
		cl.router.addRoute(
			new StaticRoute(this.uri(), new SvelteComponent(Page))
		);
	}
}