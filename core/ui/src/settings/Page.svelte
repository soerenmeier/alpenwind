<script>
	import { save as saveUser, logout } from '../api/users';
	import BackBtn from 'core-lib-ui/BackBtn';
	import FormBtn from 'core-lib-ui/FormBtn';
	import FormInput from 'core-lib-ui/FormInput';
	import { getCore } from 'core-lib';

	const cl = getCore();
	const { user, session } = cl;

	let suser = null;
	let passwordRepeat = '';
	function updateSuser(user) {
		suser = user;
		suser.password = '';
		passwordRepeat = '';
	}
	$: updateSuser($user);

	let loading = false;
	let error = '';
	async function onSaveUser(e) {
		e.preventDefault();

		if (suser.password !== passwordRepeat) {
			error = 'Passwörter stimme ned überi';
			return;
		}

		error = '';
		loading = true;
		try {
			$user = await saveUser(
				suser.name,
				suser.password,
				session.getValid().token,
			);
		} catch (e) {
			console.log('save error', e);
			error = 'Benutzer het ned chöne gspichered werde';
		}

		loading = false;
	}

	async function onLogout(e) {
		e.preventDefault();
		e.stopPropagation();

		try {
			await logout(session.getValid().token);
		} catch (e) {
			console.log('could not logout', e);
		}

		window.location.href = '/';
	}
</script>

<div id="settings">
	<header>
		<BackBtn href="/" />
	</header>

	<main>
		<section class="update-user">
			<h2>Benutzer bearbeite</h2>
			<form on:submit={onSaveUser}>
				<FormInput
					name="name"
					label="Name"
					placeholder="Name igäh"
					bind:value={suser.name}
					required
				/>
				<FormInput
					type="password"
					name="password"
					label="Passwort"
					placeholder="Passwort igäh"
					bind:value={suser.password}
				/>
				<FormInput
					type="password"
					name="password-repeat"
					label="Passwort Wiederhole"
					placeholder="Passwort igäh"
					bind:value={passwordRepeat}
				/>

				{#if error}
					<p class="error">{error}</p>
				{/if}

				<div class="btns">
					<FormBtn text="Spichere" {loading} />
					<button class="logout" on:click={onLogout}>Abmelde</button>
				</div>
			</form>
		</section>
	</main>
</div>

<style>
	header {
		margin-top: 20px;
	}

	main {
		max-width: 400px;
		margin: 0 auto;
		margin-top: 30px;
	}

	section {
		margin-top: 20px;
		padding: 20px;
		background-color: var(--gray);
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
	}

	h2 {
		margin-bottom: 30px;
	}

	form :global(input) {
		margin-bottom: 20px;
	}

	.btns {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.logout {
		background-color: transparent;
		border: none;
		color: #707070;
		transition: color 0.2s ease;
		cursor: pointer;
	}

	.logout:hover {
		color: #808080;
	}

	.error {
		margin-bottom: 15px;
	}
</style>
