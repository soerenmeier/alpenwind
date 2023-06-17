import Vault from './app.svelte';
import * as core from 'core-lib';
const { router, StaticRoute, SvelteComponent } = core.router;

function addRoutes() {
	router.addRoute(new StaticRoute('/pwvault', new SvelteComponent(Vault)));
}

export function init() {
	addRoutes();

	return {
		name: 'Passwörter'
	};
}