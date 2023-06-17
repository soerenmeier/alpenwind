<script>
	export let name = '';
	export let type = 'text';
	export let value = '';
	export let label = '';
	export let placeholder = '';
	export let required = null;
</script>

<div class="field">
	<label>{label}</label>
	{#if type === 'password'}
		<input type="password" {name} {placeholder} bind:value {required}>
	{:else}
		<input type="text" {name} {placeholder} bind:value {required}>
	{/if}
</div>

<style>
	.field {
		position: relative;
		padding-top: 5px;
	}

	label {
		position: absolute;
		top: -4px;
		left: 14px;
		padding: 0 5px;
		background-color: var(--gray);
		font-size: 13px;
		font-weight: 500;
	}

	input {
		display: block;
		width: 100%;
		padding: 8px 18px;
		border: 1px solid var(--dark-border-color);
		border-radius: 8px;
		background-color: transparent;
	}
</style>