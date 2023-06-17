<script>
	import * as core from 'core-lib';
	const { session } = core.user;
	const { open: openContextMenu } = core.contextmenu;
	import BackBtn from 'core-lib-ui/back-btn';
	import AddBtn from 'core-lib-ui/add-btn';
	import Search from 'core-lib-ui/search';
	import PasswordComp from './ui/password.svelte';
	import AddOverlay from './ui/addoverlay.svelte';
	import MasterPwOverlay from './ui/masterpwoverlay.svelte';
	import { all, EditPassword, edit, delete_ } from './lib/api.js';
	import Listeners from 'fire/util/listeners.js';
	import { sortToHigher } from 'fire/util.js';
	import { encrypt, decrypt } from './lib/crypto.js';

	window.dbgPwImport = async (p, masterPw) => {
		p.password = await encrypt(masterPw, p.password);
		
		await edit(p, session.getValid().token);
	};

	let searchVal = '';

	let passwords = [];
	async function load() {
		passwords = await all(session.getValid().token);
		sortPasswords();
	}
	load();

	let showAddOverlay = false;
	let editPassword = new EditPassword;

	let showMasterPwOverlay = false;
	let masterPwListeners = new Listeners;

	/* functions */
	function sortPasswords() {
		passwords = passwords.sort((a, b) => sortToHigher(
			a.site.toLowerCase(),
			b.site.toLowerCase()
		));
	}

	function filterPasswords(passwords, search) {
		if (!search)
			return passwords;

		// get the scores
		return passwords.map(p => [p.match(search), p])
			.filter(([score, p]) => score !== 0)
			.sort(([aScore, ap], [bScore, bp]) => sortToHigher(aScore, bScore))
			.map(([score, p]) => p);
	}

	async function requestMasterPw() {
		showMasterPwOverlay = true;

		return await new Promise(res => {
			let rm = () => {};
			rm = masterPwListeners.add(pw => {
				rm();
				showMasterPwOverlay = false;
				res(pw);
			});
		});
	}

	/* Events */
	function onAddClick() {
		showAddOverlay = true;
	}

	function onAddClose() {
		showAddOverlay = false;

		// if we edit a password that already exists
		// let's reset it
		if (editPassword.id)
			editPassword = new EditPassword;
	}

	async function onAddSubmit(e) {
		showAddOverlay = false;

		const p = editPassword;
		editPassword = new EditPassword;

		// encrypt the values
		const pw = await requestMasterPw();
		if (!pw)
			return;

		p.password = await encrypt(pw, p.password);

		try {
			const password = await edit(p, session.getValid().token);

			// check if the password already exists
			const idx = passwords.findIndex(p => p.id === password.id);
			if (idx > -1)
				passwords[idx] = password;
			else
				passwords.push(password);

			sortPasswords();
		} catch (e) {
			alert('Ds Passwort het ned chöne hinzuegfüegt werde');
			return;
		}
	}

	async function onPasswordClick(pw) {
		try {
			const masterPw = await requestMasterPw();
			if (masterPw === null)
				return;

			const realPw = await decrypt(masterPw, pw.password);
			prompt(pw.site + ': ' + pw.username, realPw);
		} catch (e) {
			console.log('error', e);
			alert('Cha ned ds passwort entschlüssle');
		}
	}

	async function onContextMenu(e, pw) {
		e.preventDefault();

		openContextMenu(e,
			[
				{ id: 'edit', text: 'Bearbeite' },
				{ id: 'delete', text: 'Lösche' }
			],
			async id => {
				if (id === 'edit') {
					editPassword = new EditPassword(pw);
					const masterPw = await requestMasterPw();
					editPassword.password = await decrypt(
						masterPw,
						editPassword.password
					);
					showAddOverlay = true;
				} else if (id === 'delete') {
					if (!confirm('Passwort würklech lösche?'))
						return;

					try {
						await delete_(pw.id, session.getValid().token);

						passwords = passwords.filter(p => p.id !== pw.id);
						sortPasswords();
					} catch (e) {
						console.log('error', e);
						alert('Cha ne ds passwort lösche');
					}
				}
			}
		);
	}
</script>

<div id="vault" class="abs-full">
	<header>
		<BackBtn href="/" />

		<div class="center">
			<Search bind:value={searchVal} />
		</div>

		<AddBtn on:click={onAddClick} />
	</header>

	<div class="list">
		{#each filterPasswords(passwords, searchVal) as password}
			<PasswordComp
				{password}
				on:click={() => onPasswordClick(password)}
				on:contextmenu={e => onContextMenu(e, password)}
			/>
		{/each}
	</div>

	{#if showAddOverlay}
		<AddOverlay
			on:close={onAddClose}
			bind:password={editPassword}
			on:submit={onAddSubmit}
		/>
	{/if}

	{#if showMasterPwOverlay}
		<MasterPwOverlay on:close={e => masterPwListeners.trigger(e.detail)} />
	{/if}
</div>

<style>
	header {
		display: grid;
		grid-template-columns: 40px 1fr 40px;
		grid-gap: 20px;
		margin-top: 20px;
	}

	.center {
		display: flex;
		justify-content: center;
	}

	.list {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		grid-gap: 20px 25px;
		padding: 60px;
	}

	@media (max-width: 2000px) {
		.list {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	@media (max-width: 1400px) {
		.list {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	@media (max-width: 1200px) {
		.list {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	@media (max-width: 900px) {
		.list {
			padding: 30px;
		}
	}

	@media (max-width: 700px) {
		.list {
			grid-template-columns: 1fr;
			grid-gap: 10px;
		}
	}

	@media (max-width: 500px) {
		.list {
			padding: 30px 20px;
		}
	}
</style>