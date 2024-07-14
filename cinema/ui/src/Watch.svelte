<script>
	import { getContext, onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import { padZero } from 'chuchi-utils';

	import BackBtn from 'core-lib-ui/BackBtn';
	import MenuBtn from 'core-lib-ui/MenuBtn';

	import { newProgressStream, bgImg } from './lib/api.ts';
	import { loadEntry } from './lib/data.ts';
	import SeekBar from './ui/seekbar.svelte';
	import Video from './ui/video.ts';
	import SelectOverlay from './ui/selectoverlay.svelte';
	import { getCore } from 'core-lib';

	/* Consts */
	const ESC = 27;
	const DOWN = 40;
	const RIGHT = 39;
	const UP = 38;
	const LEFT = 37;
	const SPACE = 32;
	const HOME = 36;
	const END = 35;
	const F_KEY = 70;
	const M_KEY = 77;

	const HIDE_MOUSE_AFTER = 1000;

	const cl = getCore();
	const { router, session } = cl;

	/* Vars */

	export let id;

	let showOverlay = true;
	let showSelectOverlay = false;
	let sendErrorShowed = false;
	let inFullscreen = false;
	let hideMouse = false;

	function onMenuClick(e) {
		showSelectOverlay = true;
		e.stopPropagation();
	}

	/// entry: WatchEntry
	let entry = null;
	let video = new Video();
	let progress = 0;
	let credits = 0;
	let watchedTime = '00:00';
	let remainingTime = '00:00';
	let progressStream = newProgressStream();
	let rmPositionUpdate = () => {};
	async function load() {
		entry = await loadEntry(id, session.getValid().token);
		await progressStream.open(session.getValid().token);
		await updateVideo();

		rmPositionUpdate = video.onPositionUpdate(onVideoPositionUpdate);
	}
	load();

	async function updateVideo() {
		const readyProm = video.waitReady();
		video.setSrc(entry.src(), entry.position());
		await readyProm;

		progress = video.progress();
		updateProgressText();
		credits = entry.creditsPercent(video.len());
	}

	/* functions */

	function bindVideo(el) {
		video.bind(el);
	}

	function updateProgressText() {
		// watchedTime
		let wSecs = video.position();
		const wMins = Math.floor(wSecs / 60);
		wSecs = Math.round(wSecs - wMins * 60);
		watchedTime = `${wMins}:${padZero(wSecs)}`;

		let rSecs = video.remainingTime();
		const rMins = Math.floor(rSecs / 60);
		rSecs = Math.round(rSecs - rMins * 60);
		remainingTime = `-${rMins}:${padZero(rSecs)}`;
	}

	let hideMouseTimeout = null;
	function waitThenHideMouse() {
		if (hideMouseTimeout) clearTimeout(hideMouseTimeout);
		hideMouseTimeout = setTimeout(() => {
			hideMouse = true;
		}, HIDE_MOUSE_AFTER);
	}

	function cancelHideMouse() {
		if (!hideMouseTimeout) return;

		clearInterval(hideMouseTimeout);
		hideMouseTimeout = null;
		hideMouse = false;
	}

	function play() {
		showOverlay = false;
		video.play();
		waitThenHideMouse();
	}

	function pause() {
		showOverlay = true;
		video.pause();
		cancelHideMouse();
	}

	function showSendError(e) {
		console.log('sendError', e);
		if (!sendErrorShowed) {
			pause();
			alert('position update het ned chöne gschickt werde');
		}
		sendErrorShowed = true;
	}

	/* events */

	let skippedCredits = false;
	async function onVideoPositionUpdate() {
		progress = video.progress();
		updateProgressText();

		const pos = video.position();

		const percent = entry.calcPercent(pos, video.len());
		entry.setProgress(percent, pos);
		try {
			entry.sendProgress(progressStream);
		} catch (e) {
			showSendError(e);
			throw e;
		}

		// let's skip the credits (only for series)
		if (entry.kind !== 'Series' || skippedCredits || !video.isPlaying())
			return;

		const creditsTime = entry.creditsTime(video.len());
		const inCreditsSkipZone = creditsTime < pos && pos < creditsTime + 1;
		const atTheEnd = video.len() - 1 < pos;

		if (!inCreditsSkipZone && !atTheEnd) return;

		skippedCredits = true;

		// let's try to skip
		const next = entry.data.nextEpisode();
		if (!next) return;

		entry.data.setEpisode(next[0], next[1]);
		entry = entry;

		await updateVideo();
		play();

		skippedCredits = false;
	}

	function onVideoClick(e) {
		pause();
	}

	// from seekbar
	function onProgressUpdate(e) {
		const nProg = e.detail;
		video.setProgress(nProg);
		progress = nProg;
		updateProgressText();
	}

	let ovCont, ovPlayBtn;
	function onOverlayClick(e) {
		if (e.target !== ovCont && e.target !== ovPlayBtn) return;

		play();
	}

	async function onSelectEpisode(e) {
		const { seasonIdx, episodeIdx } = e.detail;
		entry.data.setEpisode(seasonIdx, episodeIdx);
		entry = entry;

		await updateVideo();

		showSelectOverlay = false;
	}

	function onFullscreenClick(e) {
		if (inFullscreen) document.exitFullscreen();
		else {
			if (typeof document.body.requestFullscreen === 'function')
				document.body.requestFullscreen({ navigationUI: 'hide' });
			else
				document.body.webkitRequestFullscreen({ navigationUI: 'hide' });
		}

		// get's set by the fullscreen event
		// inFullscreen = !inFullscreen;
	}

	function onKeydown(e) {
		// only handle esc if we have the select overlay
		if (showSelectOverlay) {
			if (e.keyCode === ESC || e.keyCode === M_KEY)
				showSelectOverlay = false;

			return;
		}

		switch (e.keyCode) {
			case ESC:
				if (showOverlay) router.open('/cinema');
				else pause();
				break;
			case LEFT:
				video.reverse(10);
				break;
			case RIGHT:
				video.forward(10);
				break;
			case SPACE:
				if (showOverlay) play();
				else pause();
				break;
			case F_KEY:
				onFullscreenClick(e);
				break;
			case M_KEY:
				showSelectOverlay = true;
				break;
		}
	}

	function onMouseMove(e) {
		if (showOverlay) return;

		cancelHideMouse();
		waitThenHideMouse();
	}

	// to catch the esc click
	function onFullScreenChange(e) {
		inFullscreen = !inFullscreen;
	}

	async function onSetCompleted(ev) {
		// episode might be null
		const { season, episode, completed } = ev.detail;

		// this can only be called on a series
		const series = entry.data.inner;
		let episodes = [episode];

		if (episode === null)
			episodes = series.seasons[season].episodes.map((a, i) => i);

		let percent = completed ? 1 : 0;
		// 4hours
		let position = completed ? 14400 : 0;

		try {
			episodes.forEach(ep => {
				// position:
				series.setProgress(season, ep, percent, position);
				series.sendProgress(progressStream, season, ep);
			});
		} catch (e) {
			showSendError(e);
			throw e;
		}

		// now the video should be updated
		await updateVideo();
	}

	onDestroy(() => {
		progressStream.close();
		rmPositionUpdate();
	});
</script>

<svelte:window
	on:keydown={onKeydown}
	on:mousemove={onMouseMove}
	on:fullscreenchange={onFullScreenChange}
/>

<div id="watch" class="abs-full" class:hide-mouse={hideMouse} transition:fade>
	{#if entry}
		<div class="video abs-full" use:bindVideo on:click={onVideoClick}></div>

		{#if showOverlay}
			<div
				class="overlay abs-full"
				bind:this={ovCont}
				transition:fade={{ duration: 200 }}
				on:click={onOverlayClick}
			>
				<header>
					<BackBtn href="/cinema" />

					<h1>
						<span class="short-title">
							{entry.currentShortTitle()}
						</span>
						<span class="title">{entry.currentTitle()}</span>
					</h1>

					{#if entry.kind === 'Series'}
						<MenuBtn on:click={onMenuClick} />
					{/if}
				</header>

				<span
					class="play"
					bind:this={ovPlayBtn}
					style={bgImg('play.svg')}
				></span>

				<footer>
					<span class="text">{watchedTime}</span>
					<SeekBar
						{progress}
						{credits}
						on:update={onProgressUpdate}
					/>
					<span class="text">{remainingTime}</span>
					<span
						class="fullscreen"
						on:click={onFullscreenClick}
						style={bgImg('fullscreen.svg')}
					></span>
				</footer>
			</div>
		{/if}

		{#if showSelectOverlay && entry.kind === 'Series'}
			<SelectOverlay
				{entry}
				on:close={() => (showSelectOverlay = false)}
				on:selectEpisode={onSelectEpisode}
				on:setCompleted={onSetCompleted}
			/>
		{/if}
	{/if}
</div>

<style>
	#watch {
		overflow: hidden;
		z-index: 80;
		background-color: var(--dark-gray);
	}

	.hide-mouse {
		cursor: none;
	}

	.video {
		background-color: #000;
	}

	.video :global(video) {
		position: absolute;
		top: 50%;
		width: 100%;
		transform: translateY(-50%);
	}

	.overlay {
		z-index: 20;
		background-color: rgba(0, 0, 0, 0.4);
	}

	header {
		display: grid;
		height: 43px;
		margin-top: 20px;
		grid-template-columns: 40px 1fr 40px;
		grid-gap: 20px;
		align-items: center;
	}

	h1 {
		overflow: hidden;
		white-space: nowrap;
	}

	.short-title {
		display: none;
	}

	.play {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 100px;
		height: 100px;
		background-size: contain;
		background-repeat: no-repeat;
		background-position: center;
		transform: translate(-50%, -50%);
		cursor: pointer;
	}

	footer {
		position: absolute;
		display: grid;
		bottom: 40px;
		width: 100%;
		padding: 0 40px;
		grid-template-columns: auto 1fr auto auto;
		grid-gap: 15px;
		align-items: center;
	}

	footer .text {
		user-select: none;
		min-width: 53px;
		text-align: center;
	}

	.fullscreen {
		position: relative;
		display: block;
		width: 24px;
		height: 24px;
		margin-left: 10px;
		align-items: center;
		cursor: pointer;
		transform: scale(1);
		transition: transform 0.2s ease;
	}

	.fullscreen:hover {
		transform: scale(1.2);
	}

	@media (max-width: 900px) {
		.title {
			display: none;
		}

		.short-title {
			display: block;
		}

		footer {
			bottom: 30px;
			padding: 0 30px;
		}
	}

	@media (max-width: 500px) {
		footer {
			bottom: 20px;
			padding: 0 20px;
		}
	}
</style>
