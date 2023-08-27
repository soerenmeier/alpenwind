<script>
	import { getContext } from 'svelte';
	import { login } from '../api/users.js';

	const cl = getContext('cl');
	const { session, user } = cl;

	let username = '';
	let password = '';
	let error = '';

	async function submitLogin() {
		error = '';
		try {
			const loginRes = await login(username, password);
			$session = loginRes.session;
			$user = loginRes.user;
			console.log('logged in');
		} catch (e) {
			console.log('login error', e);

			const kind = e.kind ?? e.message;
			if (kind === 'LoginIncorrect')
				error = 'Email or passwort fausch';
			else
				error = 'Login informationen hei ned chöne verschickt werde';

			password = '';
		}
	}
</script>

<div id="login" class="abs-full bg-image">

	<div class="box">
		<div class="inner-box">
			<h1>Dihei</h1>

			<form on:submit|preventDefault={submitLogin}>
				<input
					type="text"
					name="username"
					required
					bind:value={username}
					placeholder="Benutzername"
				>
				<input
					type="password"
					name="password"
					required
					bind:value={password}
					placeholder="Passwort"
				>
				{#if error}
					<div class="error-box">{error}</div>
				{/if}
				<button class="action-btn">Amelde</button>
			</form>
		</div>
	</div>

</div>

<style>
	#login {
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.box {
		width: 400px;
		padding: 50px 40px;
		border: 1px solid var(--blur-border-color);
		backdrop-filter: blur(20px) brightness(0.6);
		border-radius: 10px;
	}

	h1 {
		margin-bottom: 40px;
	}

	input {
		display: block;
		width: 100%;
		margin-bottom: 20px;
		padding: 9px 15px;
		background-color: transparent;
		border: 1px solid rgba(255, 255, 255, .2);
		border-radius: 8px;
	}

	input::placeholder {
		color: #828282;
	}

	.error-box {
		margin-bottom: 20px;
	}

	@media (max-width: 500px) {
		.box {
			display: flex;
			width: 100%;
			height: 100%;
			padding: 20px;
			align-items: center;
			justify-content: center;
			border: none;
			border-radius: 0;
		}

		.inner-box {
			width: 100%;
		}
	}
</style>