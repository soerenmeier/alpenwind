import Cinema from './app.svelte';
import Watch from './watch.svelte';
import * as core from 'core-lib';
const { router, Route, StaticRoute, SvelteComponent } = core.router;

const WATCH_URI = '/cinema/watch/';

class WatchRoute extends Route {
	constructor() {
		super();

		// the routing system isn't great but will work
		this._props = {
			id: ''
		};
	}

	check(req) {
		if (!req.uri.startsWith(WATCH_URI))
			return false;

		const id = req.uri.substr(WATCH_URI.length);
		if (id.length < 14 || id.length > 16)
			return false;

		this._props = {
			id: id.substr(0, 14)
		};

		return true;
	}

	attachComponent(el) {
		return (new SvelteComponent(Watch, this._props)).attach(el);
	}
}

function addRoutes() {
	router.addRoute(new StaticRoute('/cinema', new SvelteComponent(Cinema)));
	router.addRoute(new WatchRoute);
}


export function init() {
	addRoutes();

	return {
		name: 'Cinema'
	};
}