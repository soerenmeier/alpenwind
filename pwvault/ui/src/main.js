import Vault from './app.svelte';
import * as core from 'core-lib';
const { StaticRoute, SvelteComponent } = core.router;

function addRoutes(router) {
	router.addRoute(new StaticRoute('/pwvault', new SvelteComponent(Vault)));
}

export function init(cl) {
	addRoutes(cl.router);

	return {
		name: 'Passwörter'
	};
}