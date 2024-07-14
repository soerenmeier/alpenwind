<script>
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	export let progress = 0;
	// also in percent but from the right (so meaning how long are the credits)
	export let credits = 0;

	let cont;

	let offset = 0;
	let width = 0;

	let down = false;
	function onMouseDown(e) {
		e.preventDefault();
		down = true;

		const cOff = cont.getBoundingClientRect();
		offset = cOff.left;
		width = cont.clientWidth;

		update(e.clientX);
	}

	function onTouchStart(e) {
		const touch = e.targetTouches[0];
		e.clientX = touch.clientX;
		onMouseDown(e);
	}

	function onMouseMove(e) {
		if (!down) return;

		update(e.clientX);
	}

	function onTouchMove(e) {
		onMouseMove(e.targetTouches[0]);
	}

	function onMouseUp(e) {
		if (!down) return;

		down = false;
		e.preventDefault();
		e.stopPropagation();
	}

	function onTouchEnd(e) {
		onMouseUp(e);
	}

	function update(clientX) {
		let off = clientX - offset;
		off /= width;
		progress = Math.max(Math.min(off, 1), 0);

		dispatch('update', progress);
	}
</script>

<svelte:window
	on:mousemove={onMouseMove}
	on:mouseup={onMouseUp}
	on:touchmove={onTouchMove}
	on:touchend={onTouchEnd}
/>

<div
	class="seekbar"
	style="--progress: {progress * 100}%; --credits: {credits * 100}%"
	on:mousedown={onMouseDown}
	on:touchstart={onTouchStart}
	bind:this={cont}
	role="none"
>
	<span class="bar"></span>
	<span class="cursor"></span>
</div>

<style>
	.seekbar {
		position: relative;
		height: 20px;
		cursor: pointer;
	}

	.cursor {
		position: absolute;
		top: 3px;
		left: calc(var(--progress) - 7px);
		width: 14px;
		height: 14px;
		background-color: var(--red);
		border-radius: 50%;
	}

	.bar {
		position: absolute;
		top: 9px;
		width: 100%;
		height: 2px;
		background-color: #fff;
		transition: transform 0.2s ease;
	}

	.bar::before,
	.bar::after {
		content: '';
		position: absolute;
		top: 0;
		height: 100%;
	}

	.bar::before {
		right: 0;
		width: var(--credits);
		background-color: #f3f36a;
	}

	.bar::after {
		left: 0;
		width: var(--progress);
		background-color: var(--red);
	}

	.seekbar:hover .bar {
		transform: scaleY(2);
	}
</style>
