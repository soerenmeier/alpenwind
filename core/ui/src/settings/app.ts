import App from '../lib/App.js';
import Page from './Page.svelte';
import * as core from 'core-lib';
// @ts-ignore
import settingsIcon from '../../assets/settings/icon.png?url';
const { SvelteComponent } = core.router;

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
		cl.router.register(this.uri(), () => new SvelteComponent(Page));
	}
}
