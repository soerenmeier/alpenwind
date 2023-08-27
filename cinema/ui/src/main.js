import Cinema from './app.svelte';
import Watch from './watch.svelte';
import * as core from 'core-lib';
const { router, StaticRoute, SvelteComponent } = core.router;

function addRoutes() {
	router.addRoute(new StaticRoute('/cinema', new SvelteComponent(Cinema)));
	router.addRoute(new StaticRoute(
		/^\/cinema\/watch\/(?<id>[A-Za-z0-9-_]{14})$/,
		new SvelteComponent(Watch)
	));
}


export function init() {
	addRoutes();

	return {
		name: 'Cinema'
	};
}