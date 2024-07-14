import './app.css';

import { router, newCore } from 'core-lib';
const { Route, SvelteComponent } = router;
import App from './App.svelte';
import Apps from './pages/Apps.svelte';
import { getContext } from 'svelte';

function main() {
	const cl = newCore({});

	cl.router.register('/', async () => new SvelteComponent(Apps));

	// setup core.api
	// @ts-ignore
	window.API_SERVER_ADDR = import.meta.env.SERVER_ADDR;

	const app = new App({
		target: document.body,
	});
}
main();
