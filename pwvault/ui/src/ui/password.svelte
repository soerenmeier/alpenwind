<script>
	export let password;
</script>

<button type="button" on:click on:contextmenu>
	<span class="figure">
		<img src={password.favicon()} alt={password.site} aria-hidden={true} />
	</span>

	<span class="texts">
		<span class="title">{password.site}</span>
		<span class="user">{password.username}</span>
	</span>
</button>

<style>
	button {
		display: flex;
		padding: 7px 15px;
		gap: 30px;
		align-items: center;
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
		background-color: var(--gray);
		cursor: pointer;
		appearance: none;
		text-align: left;
		color: inherit;
		transition: background-color 0.2s ease;
	}

	button:hover {
		background-color: #2d2d2d;
	}

	.figure {
		display: block;
	}

	img {
		width: 32px;
	}

	.texts {
		width: 100%;
	}

	.texts span {
		display: block;
	}

	.title {
		font-weight: 500;
		margin-bottom: -2px;
	}

	.user {
		color: var(--real-light-gray);
	}
</style>
