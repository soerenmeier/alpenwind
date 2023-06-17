<script>
	export let value;
</script>

<div class="search">
	<input
		type="text"
		name="search"
		placeholder="Sueche"
		bind:value
	>
</div>

<style>
	input {
		display: block;
		width: 350px;
		padding: 9px 15px;
		background-color: transparent;
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
		background-color: var(--gray);
		text-align: center;
	}

	input::placeholder {
		color: #fff;
	}

	@media (max-width: 500px) {
		.search {
			width: 100%;
		}

		input {
			width: 100%;
		}
	}
</style>