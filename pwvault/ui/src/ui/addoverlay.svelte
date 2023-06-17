<script>
	import { createEventDispatcher } from 'svelte';
	import { cubicInOut } from 'svelte/easing';
	import * as core from 'core-lib';
	import CloseBtn from 'core-lib-ui/close-btn';
	import BlueFormBtn from 'core-lib-ui/blue-form-btn';
	import Field from 'core-lib-ui/form-input';

	/* consts */
	const dispatch = createEventDispatcher();

	function backgroundColor(node, params) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `background-color: rgba(0,0,0,${t * 0.4})`
		};
	}

	function slide(node, params) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `transform: translateX(${(1 - t) * 100}%)`
		};
	}

	/* Vars */
	// should be an EditPasswors
	export let password;

	/// Event handlers
	let ovCont;
	function onOverlayClick(e) {
		if (e.target !== ovCont)
			return;

		dispatch('close');
	}

	function onCloseClick(e) {
		dispatch('close');
	}

	function onSubmit(e) {
		e.preventDefault();

		dispatch('submit');
	}
</script>

<div
	class="add-overlay abs-full"
	transition:backgroundColor
	bind:this={ovCont}
	on:click={onOverlayClick}
>
	<div class="bar" transition:slide>
		<CloseBtn on:click={onCloseClick} />

		{#if password.id}
			<h2>Passwort bearbeite</h2>
		{:else}
			<h2>Neues Passwort</h2>
		{/if}

		<form class="fields" on:submit={onSubmit}>
				<Field
					name="site"
					label="Site"
					placeholder="Site igäh"
					bind:value={password.site}
					required
				/>
				<Field
					name="domain"
					label="Site link"
					placeholder="Site link igäh"
					bind:value={password.domain}
					required
				/>
				<Field
					name="username"
					label="Benutzername"
					placeholder="Benutzername igäh"
					bind:value={password.username}
					required
				/>
				<Field
					type="password"
					name="password"
					label="Passwort"
					placeholder="Passwort igäh"
					bind:value={password.password}
					required
				/>

				{#if password.id}
					<BlueFormBtn text="Passwort spichere" />
				{:else}
					<BlueFormBtn text="Neus Passwort hinzuefüege" />
				{/if}
		</form>
	</div>
</div>

<style>
	.add-overlay {
		position: fixed;
		background-color: rgba(0, 0, 0, .4);
		z-index: 30;
		overflow: hidden;
	}

	.bar {
		position: absolute;
		right: 0;
		height: 100%;
		width: 400px;
		padding: 27px 20px 20px 20px;
		background-color: var(--gray);
		border-left: 1px solid var(--dark-border-color);
	}

	.bar :global(.close-btn) {
		position: absolute;
		top: 20px;
		right: 100%;
	}

	.fields {
		margin-top: 45px;
	}

	.fields :global(> *) {
		margin-bottom: 20px;
	}

	@media (max-width: 500px) {
		.bar {
			width: calc(100% - 40px);
		}
	}
</style>