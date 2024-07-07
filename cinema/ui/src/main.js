import Cinema from './App.svelte';
import Watch from './Watch.svelte';
import * as core from 'core-lib';
const { SvelteComponent } = core.router;

function addRoutes(router) {
	router.register('/cinema', () => new SvelteComponent(Cinema));
	router.register(
		/^\/cinema\/watch\/(?<id>[A-Za-z0-9-_]{14})$/,
		() => new SvelteComponent(Watch),
	);
}

export function init(cl) {
	addRoutes(cl.router);

	return {
		name: 'Cinema',
	};
}
