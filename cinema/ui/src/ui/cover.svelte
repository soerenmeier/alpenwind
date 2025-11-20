<script>
	export let src;
	export let alt = 'Cover';
	export let percent = 0;
	export let lazy = false;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<figure class="cover" style:--percent={percent} on:click on:contextmenu>
	<img {src} {alt} loading={lazy ? 'lazy' : 'eager'} />
</figure>

<style>
	.cover {
		position: relative;
		width: 100%;
		aspect-ratio: 0.63;
		background-color: var(--gray);
	}

	img {
		display: block;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.cover::after {
		content: '';
		position: absolute;
		bottom: 0;
		height: 3px;
		width: calc(var(--percent) * 100%);
		background-color: var(--red);
	}
</style>
