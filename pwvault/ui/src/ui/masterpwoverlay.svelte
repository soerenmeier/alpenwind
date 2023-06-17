<script>
	import { onMount, createEventDispatcher } from 'svelte';
	import { fade } from 'svelte/transition';
	import CloseBtn from 'core-lib-ui/close-btn';

	/* consts */
	const dispatch = createEventDispatcher();

	/* Vars */
	let password = '';
	let field;

	/// Event handlers
	let ovCont;
	function onOverlayClick(e) {
		if (e.target !== ovCont)
			return;

		dispatch('close', null);
	}

	function onSubmit(e) {
		e.preventDefault();
		dispatch('close', password);
	}

	onMount(() => {
		field.focus();
	});
</script>

<div
	class="masterpw-overlay abs-full"
	transition:fade={{duration: 200}}
	bind:this={ovCont}
	on:click={onOverlayClick}
>
	<form class="fields" on:submit={onSubmit}>
			<input
				bind:this={field}
				type="password"
				name="masterpw"
				placeholder="Master Passwort igäh"
				required
				bind:value={password}
			>
			<button>Los</button>
	</form>
</div>

<style>
	.masterpw-overlay {
		position: fixed;
		display: flex;
		background-color: rgba(0, 0, 0, .6);
		z-index: 40;
		align-items: center;
		justify-content: center;
	}

	.fields {
		background-color: var(--gray);
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
	}

	input {
		background-color: transparent;
		border: none;
		padding: 8px 18px;
	}

	button {
		background-color: #3c7bd5;
		border: none;
		padding: 8px 18px;
		color: var(--white);
		cursor: pointer;
		transition: background-color .2s ease;
		border-top-right-radius: 6px;
		border-bottom-right-radius: 6px;
	}

	button:hover {
		background-color: #2465c1;
	}
</style>