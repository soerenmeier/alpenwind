<script>
	import { getAllContexts } from 'svelte';
	import { router, getCore } from 'core-lib';
	const { SvelteComponent } = router;
	import ContextMenuOverlay from 'core-lib-ui/ContextMenu';
	import { loginByToken } from './api/users';
	import { loadApps } from './lib/apps';
	import Login from './pages/Login.svelte';

	const cl = getCore();
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

	// let component = null;
	// let loaded = false;

	let cont;
	let destroyComp = () => {};

	async function load() {
		await Promise.all([loadSession(), loadApps(cl)]);

		cl.router.onRoute(async (req, route, routing) => {
			destroyComp();

			if (!session.get()) {
				await routing.dataReady();

				const login = new SvelteComponent(Login);
				destroyComp = login.attach(cont, {}, allContext);

				routing.domReady();
				return;
			}

			if (!route) {
				await routing.dataReady();
				cont.innerText = 'not found';
				routing.domReady();
				return;
			}

			const comp = await route.load(req);
			await routing.dataReady();
			destroyComp = comp.attach(cont, route.toProps(req), allContext);
			routing.domReady();
		});
		cl.router.initClient();
	}
	load();
</script>

<div class="cont" bind:this={cont}></div>

<ContextMenuOverlay />
