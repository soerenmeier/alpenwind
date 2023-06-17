import './app.css';

import * as core from 'core-lib';
const { router, StaticRoute, SvelteComponent } = core.router;
import Api from 'fire/api/api.js';
import Stream from 'fire/api/stream.js';
import App from './app.svelte';
import Apps from './pages/apps.svelte';

router.addRoute(new StaticRoute('/', new SvelteComponent(Apps)));

// setup core.api
const serverAddr = import.meta.env.SERVER_ADDR;
core.api.init(serverAddr);

const app = new App({
	target: document.body
});

export default app;