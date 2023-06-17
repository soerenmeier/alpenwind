<span class="close-btn" on:click>
	<span class="arrow"></span>
</span>

<style>
	.close-btn {
		width: 40px;
		height: 43px;
		background-color: var(--gray);
		border: 1px solid var(--dark-border-color);
		border-right: 0;
		border-radius: 8px 0 0 8px;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.close-btn:hover {
		background-color: var(--light-gray);
	}

	.arrow {
		position: absolute;
		top: 15px;
		left: 12px;
		width: 11px;
		height: 11px;
		border-top: 2px solid #fff;
		border-right: 2px solid #fff;
		border-radius: 1px;
		transform: rotate(45deg);
	}
</style>