<script>
	import { getContext, getAllContexts } from 'svelte';
	import { router } from 'core-lib';
	const { SvelteComponent } = router;
	import ContextMenuOverlay from 'core-lib-ui/contextmenu';
	import { loginByToken } from './api/users.js';
	import { loadApps } from './lib/apps.js';
	import Login from './pages/login.svelte';

	const cl = getContext('cl');
	const { session, user } = cl;

	const allContext = getAllContexts();

	async function loadSession() {
		const sess = session.get();
		console.log('login by token', sess);
		if (sess) {
			try {
				// need to login by token
				const loginData = await loginByToken(sess.token);
				$session = loginData.session;
				$user = loginData.user;
			} catch (e) {
				$session = null;
				$user = null;
			}
		}
	}

	let route = null;
	let loaded = false;
	async function load() {
		await Promise.all([loadSession(), loadApps(cl)]);

		cl.router.onRouteChange((rout, req) => {
			console.log('route change', req);
			route = rout;
		});
		cl.router.init();

		loaded = true;
	}
	load();

	let cont;
	let destroyComp = () => {};
	function handleRoutes(session, route, loaded) {
		if (!loaded)
			return;

		destroyComp();
		if (!session) {
			// show login
			const login = new SvelteComponent(Login);
			destroyComp = login.attach(cont, {}, allContext);
		} else if (route) {
			destroyComp = route.attachComponent(cont, allContext);
		} else {
			cont.innerText = 'not found';
		}
	}
	$: handleRoutes($session, route, loaded);
</script>


<div class="cont" bind:this={cont}></div>

<ContextMenuOverlay />