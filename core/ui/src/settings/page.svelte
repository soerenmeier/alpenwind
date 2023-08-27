<script>
	import { getContext } from 'svelte';
	import { save as saveUser, logout } from '../api/users.js';
	import BackBtn from 'core-lib-ui/back-btn';
	import BlueFormBtn from 'core-lib-ui/blue-form-btn';
	import FormInput from 'core-lib-ui/form-input';

	const cl = getContext('cl');
	const { user, session } = cl;

	let suser = null;
	function updateSuser(user) {
		suser = user;
		suser.password = '';
	}
	$: updateSuser($user);

	async function onSaveUser(e) {
		e.preventDefault();

		try {
			$user = await saveUser(
				suser.name,
				suser.password,
				session.getValid().token
			);
		} catch (e) {
			console.log('save error', e);
			alert('Benutzer het ned chöne gspichered werde');
		}
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
					name="password"
					label="Passwort"
					placeholder="Passwort igäh"
					bind:value={suser.password}
				/>

				<div class="btns">
					<BlueFormBtn text="Spichere" />
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
		transition: color .2s ease;
		cursor: pointer;
	}

	.logout:hover {
		color: #808080;
	}
</style>