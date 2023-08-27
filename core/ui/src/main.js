import './app.css';

import CoreLib, { router } from 'core-lib';
const { StaticRoute, SvelteComponent } = router;
import Api from 'fire/api/api.js';
import Stream from 'fire/api/stream.js';
import App from './app.svelte';
import Apps from './pages/apps.svelte';

function main() {
	const cl = new CoreLib;

	cl.router.addRoute(new StaticRoute('/', new SvelteComponent(Apps)));

	// setup core.api
	window.API_SERVER_ADDR = import.meta.env.SERVER_ADDR;

	const context = new Map;
	context.set('cl', cl);

	const app = new App({
		target: document.body,
		context
	});
}
main();