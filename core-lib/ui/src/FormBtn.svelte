<script>
	export let text = '';
	export let loading = false;
	export let color = 'blue';
</script>

<button on:click class:loading class="color-{color}">
	<span class="text">{text}</span>
</button>

<style>
	button {
		position: relative;
		border: none;
		padding: 8px 18px;
		color: var(--white);
		border-radius: 8px;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.color-blue {
		background-color: #3c7bd5;
	}

	.color-blue:hover {
		background-color: #2465c1;
	}

	.color-red {
		background-color: var(--red);
	}

	.color-red:hover {
		background-color: var(--dark-red);
	}

	.loading .text {
		opacity: 0;
	}

	.loading::after {
		content: '';
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 20px;
		height: 20px;
		border: 2px solid var(--white);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: translate(-50%, -50%) rotate(360deg);
		}
	}
</style>
