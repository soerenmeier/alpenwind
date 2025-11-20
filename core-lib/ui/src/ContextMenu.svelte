<script>
	import { getCore } from 'core-lib';

	const cl = getCore();
	const currentOpts = cl.contextMenu.currentOpts;

	// checkout ContextMenu.ts file for type info
	let ctx = null;
	let opts = [];
	let pos = [];
	let overlayEl = null;

	function update(nCtx) {
		if (!nCtx) {
			ctx = null;
			return;
		}

		ctx = nCtx;
		opts = ctx[1];
		const ev = ctx[0];
		pos = [ev.clientX, ev.clientY];
	}
	$: update($currentOpts);

	function close(id = null) {
		ctx[2](id);
	}

	function positionBox(el, opts) {
		const { x, y } = opts;

		const maxX = window.innerWidth - el.offsetWidth;
		const maxY = window.innerHeight - el.offsetHeight;

		el.style.top = `${Math.min(y, maxY)}px`;
		el.style.left = `${Math.min(x, maxX)}px`;
	}

	function onClick(e) {
		if (e.target !== overlayEl) return;

		close();
	}
</script>

{#if ctx}
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<div bind:this={overlayEl} class="overlay abs-full" on:click={onClick}>
		<div class="box" use:positionBox={{ x: pos[0], y: pos[1] }}>
			{#each opts as opt}
				<button on:click={() => close(opt.id)}>{opt.text}</button>
			{/each}
			<button on:click={() => close()}>Abbreche</button>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		background-color: rgba(0, 0, 0, 0.4);
		z-index: 90;
	}

	.box {
		position: absolute;
		background-color: var(--white);
		border: 1px solid var(--gray);
		border-radius: 5px;
		padding: 5px 0;
	}

	button {
		display: block;
		width: 100%;
		text-align: left;
		padding: 5px 15px;
		border: none;
		color: var(--gray);
		background-color: transparent;
		cursor: pointer;
	}

	button:hover {
		background-color: #eee;
	}
</style>
