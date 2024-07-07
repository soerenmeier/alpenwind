<script>
	import { getContext, createEventDispatcher } from 'svelte';
	import { slide as svelteSlide } from 'svelte/transition';
	import { cubicInOut } from 'svelte/easing';
	import CloseBtn from 'core-lib-ui/CloseBtn';
	import { getCore } from 'core-lib';

	/* consts */
	const dispatch = createEventDispatcher();
	const { contextMenu } = getCore();
	// const openCtx = cl.contextMenu.open;

	function backgroundColor(node, params) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `background-color: rgba(0,0,0,${t * 0.4})`,
		};
	}

	function slide(node, params) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `transform: translateX(${(1 - t) * 100}%)`,
		};
	}

	/* Vars */
	// should always contain an entry which is a series
	export let entry;

	let openSeason = entry.data.cSeason;

	/// Event handlers
	let sovCont;
	function onSelectOverlayClick(e) {
		if (e.target !== sovCont) return;

		dispatch('close');
	}

	function onCloseClick(e) {
		dispatch('close');
	}

	function onSelectEpisode(seasonIdx, episodeIdx) {
		dispatch('selectEpisode', { seasonIdx, episodeIdx });
	}

	/// episode might be null
	function setCompleted(seasonIdx, episodeIdx, completed) {
		dispatch('setCompleted', {
			season: seasonIdx,
			episode: episodeIdx,
			completed,
		});
		entry = entry;
	}

	/// episode might be null
	function ctxMenu(e, seasonIdx, episodeIdx) {
		e.preventDefault();
		e.stopPropagation();

		contextMenu.open(
			e,
			[
				{ id: 'reset', text: 'Nonid gluegt' },
				{ id: 'setCompleted', text: 'Scho gluegt' },
			],
			id => {
				if (id === 'reset') setCompleted(seasonIdx, episodeIdx, false);
				else if (id === 'setCompleted')
					setCompleted(seasonIdx, episodeIdx, true);
				// else if (id === '')
			},
		);
	}
</script>

<div
	class="select-overlay abs-full"
	transition:backgroundColor
	bind:this={sovCont}
	on:click={onSelectOverlayClick}
>
	<div class="selection" transition:slide>
		<CloseBtn on:click={onCloseClick} />

		<div class="scroller">
			<h2>Episode uswähle</h2>

			<div class="list">
				{#each entry.data.seasons() as season, idx}
					<div class="season">
						<span
							class="entry"
							style="--progress: {season.totalPercent() * 100}%"
							on:click={() => {
								openSeason = openSeason === idx ? null : idx;
							}}
							on:contextmenu={e => ctxMenu(e, idx, null)}
						>
							<span>{season.title(idx)}</span>
						</span>

						{#if openSeason === idx}
							<div
								class="season-list"
								transition:svelteSlide={{ duration: 200 }}
							>
								{#each season.episodes as episode, eId}
									<span
										class="entry"
										style="--progress: {episode.percent() *
											100}%"
										on:click={() =>
											onSelectEpisode(idx, eId)}
										on:contextmenu={e =>
											ctxMenu(e, idx, eId)}
									>
										<span>{episode.title(eId)}</span>
									</span>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	</div>
</div>

<style>
	.select-overlay {
		background-color: rgba(0, 0, 0, 0.4);
		z-index: 30;
	}

	.selection {
		position: absolute;
		right: 0;
		height: 100%;
		width: 400px;
		background-color: var(--gray);
		border-left: 1px solid var(--dark-border-color);
	}

	.selection :global(.close-btn) {
		position: absolute;
		top: 20px;
		right: 100%;
	}

	.scroller {
		height: 100%;
		overflow-y: auto;
	}

	h2 {
		margin: 27px 20px 10px 20px;
	}

	.entry {
		position: relative;
		display: block;
		padding: 10px 20px;
		cursor: pointer;
		background-color: transparent;
		transition: background-color 0.2s ease;
	}

	.entry:hover {
		background-color: var(--light-gray);
	}

	.entry::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		width: var(--progress);
		height: 100%;
		background-color: var(--red);
	}

	.entry span {
		position: relative;
		z-index: 1;
	}

	.season-list .entry {
		padding: 10px 40px;
	}

	@media (max-width: 500px) {
		.selection {
			width: calc(100% - 40px);
		}
	}
</style>
