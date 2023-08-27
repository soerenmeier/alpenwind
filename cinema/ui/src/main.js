import Cinema from './app.svelte';
import Watch from './watch.svelte';
import * as core from 'core-lib';
const { StaticRoute, SvelteComponent } = core.router;

function addRoutes(router) {
	router.addRoute(new StaticRoute('/cinema', new SvelteComponent(Cinema)));
	router.addRoute(new StaticRoute(
		/^\/cinema\/watch\/(?<id>[A-Za-z0-9-_]{14})$/,
		new SvelteComponent(Watch)
	));
}


export function init(cl) {
	addRoutes(cl.router);

	return {
		name: 'Cinema'
	};
}