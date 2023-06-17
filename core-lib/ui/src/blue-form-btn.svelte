<script>
	export let text = '';
</script>

<button on:click>{text}</button>

<style>
	button {
		background-color: #3c7bd5;
		border: none;
		padding: 8px 18px;
		color: var(--white);
		border-radius: 8px;
		cursor: pointer;
		transition: background-color .2s ease;
	}

	button:hover {
		background-color: #2465c1;
	}
</style>