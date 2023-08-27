<script>
	import { getContext } from 'svelte';

	const cl = getContext('cl');
	const { router } = cl;

	export let href = '';
	export let onlyHref = false;

	function onClick(e) {
		if (router.canGoBack() && !onlyHref) {
			e.preventDefault();
			e.stopPropagation();
			router.back();
		}
	}
</script>

<a {href} class="back-btn" on:click={onClick}>
	<span class="arrow"></span>
</a>

<style>
	.back-btn {
		position: relative;
		display: block;
		width: 40px;
		height: 43px;
		background-color: var(--gray);
		border: 1px solid var(--dark-border-color);
		border-left: none;
		border-radius: 0 8px 8px 0;
		cursor: pointer;
	}

	.back-btn:hover {
		background-color: var(--light-gray);
	}

	.arrow {
		position: absolute;
		top: 15px;
		left: 17px;
		width: 11px;
		height: 11px;
		border-top: 2px solid #fff;
		border-left: 2px solid #fff;
		border-radius: 1px;
		transform: rotate(-45deg);
	}
</style>