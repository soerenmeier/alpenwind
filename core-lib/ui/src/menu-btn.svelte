<script>
</script>

<span class="menu-btn" on:click>
	<span class="burger">
		<span></span>
	</span>
</span>

<style>
	.menu-btn {
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

	.menu-btn:hover {
		background-color: var(--light-gray);
	}

	.burger {
		position: absolute;
		top: 14px;
		left: 15px;
		height: 14px;
		width: 10px;
	}

	.burger::before, .burger::after {
		content: "";
	}

	.burger::before, .burger span, .burger::after {
		position: absolute;
		width: 100%;
		height: 2px;
		background-color: #fff;
	}

	.burger span {
		top: 6px;
	}

	.burger::after {
		bottom: 0;
	}
</style>