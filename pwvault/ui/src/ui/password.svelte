<script>
	export let password;
</script>

<article on:click on:contextmenu>
	<figure>
		<img src={password.favicon()}>
	</figure>
	<div class="texts">
		<h3>{password.site}</h3>
		<p class="user">{password.username}</p>
	</div>
</article>

<style>
	article {
		display: flex;
		padding: 7px 15px;
		gap: 30px;
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
		align-items: center;
		background-color: var(--gray);
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	article:hover {
		background-color: #2d2d2d;
	}

	img {
		width: 32px;
	}

	.texts {
		width: 100%;
	}

	h3 {
		font-weight: 500;
		margin-bottom: -2px;
	}

	.user {
		color: var(--real-light-gray);
	}
</style>