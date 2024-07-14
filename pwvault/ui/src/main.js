import Vault from './App.svelte';
import * as core from 'core-lib';
const { SvelteComponent } = core.router;

function addRoutes(router) {
	router.register('/pwvault', () => new SvelteComponent(Vault));
}

export function init(cl) {
	addRoutes(cl.router);

	return {
		name: 'Passwörter',
	};
}
