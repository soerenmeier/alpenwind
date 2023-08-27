<script>
	export let src;
	export let alt = 'Cover';
	export let percent = 0;
</script>

<figure class="cover" style:--percent={percent} on:click on:contextmenu>
	<img {src} {alt}>
</figure>

<style>
	.cover {
		position: relative;
		width: 100%;
		background-image: var(--poster);
		background-repeat: no-repeat;
		background-size: cover;
		background-position: center;
		aspect-ratio: 0.63;
	}

	img {
		display: block;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.cover::after {
		content: "";
		position: absolute;
		bottom: 0;
		height: 3px;
		width: calc(var(--percent) * 100%);
		background-color: var(--red);
	}
</style>