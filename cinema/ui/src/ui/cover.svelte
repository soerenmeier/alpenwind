<script>
	export let src;
	export let percent = 0;
</script>

<div
	class="cover"
	style="--poster: url('{src.replace(/'/, '%27')}'); --percent: {percent * 100}%"
	on:click
></div>

<style>
	.cover {
		position: relative;
		width: 100%;
		background-image: var(--poster);
		background-repeat: no-repeat;
		background-size: cover;
		background-position: center;
	}

	.cover::before {
		content: "";
		display: block;
		width: 100%;
		padding-bottom: 158.73%;
	}

	.cover::after {
		content: "";
		position: absolute;
		bottom: 0;
		height: 3px;
		width: var(--percent);
		background-color: var(--red);
	}
</style>