
<span class="back-btn" on:click>
	<span class="plus"></span>
</span>

<style>
	.back-btn {
		position: relative;
		display: block;
		width: 40px;
		height: 43px;
		background-color: var(--gray);
		border: 1px solid var(--dark-border-color);
		border-right: none;
		border-radius: 8px 0 0 8px;
		cursor: pointer;
	}

	.back-btn:hover {
		background-color: var(--light-gray);
	}

	.plus {
		position: absolute;
		top: calc(100% / 2 - 8px);
		left: calc(100% / 2 - 8px);
		height: 16px;
		width: 16px;
	}

	.plus::before, .plus::after {
		position: absolute;
		content: '';
		background-color: #fff;
	}

	.plus::before {
		left: calc(100% / 2 - 1px);
		width: 2px;
		height: 100%;
	}

	.plus::after {
		top: calc(100% / 2 - 1px);
		width: 100%;
		height: 2px;
	}
</style>