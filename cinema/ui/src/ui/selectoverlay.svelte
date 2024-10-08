<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { slide as svelteSlide } from 'svelte/transition';
	import { cubicInOut } from 'svelte/easing';
	import CloseBtn from 'core-lib-ui/CloseBtn';
	import { getCore } from 'core-lib';
	import { SeriesEntry } from '../lib/data';
	import { Episode, Season } from '../lib/api';
	import { padZero } from 'chuchi-utils';

	/* consts */
	const dispatch = createEventDispatcher();
	const { contextMenu } = getCore();
	// const openCtx = cl.contextMenu.open;

	function backgroundColor(
		node: HTMLElement,
		params?: {
			delay?: number;
			duration?: number;
			easing?: (t: number) => number;
		},
	) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `background-color: rgba(0,0,0,${t * 0.4})`,
		};
	}

	function slide(
		node: HTMLElement,
		params?: {
			delay?: number;
			duration?: number;
			easing?: (t: number) => number;
		},
	) {
		return {
			delay: params.delay ?? 0,
			duration: params.duration ?? 400,
			easing: params.easing ?? cubicInOut,
			css: (t, u) => `transform: translateX(${(1 - t) * 100}%)`,
		};
	}

	/* Vars */
	// should always contain an entry which is a series
	export let entry: SeriesEntry;

	let openSeason = entry.cSeason;

	/// Event handlers
	let sovCont;
	function onSelectOverlayClick(e) {
		if (e.target !== sovCont) return;

		dispatch('close');
	}

	function onCloseClick(e) {
		dispatch('close');
	}

	function onSelectEpisode(episodeId: string) {
		dispatch('selectEpisode', episodeId);
	}

	/// episode might be null
	function setCompleted(
		seasonIdx: number,
		episodeId?: string,
		completed: boolean = true,
	) {
		dispatch('setCompleted', {
			season: seasonIdx,
			episode: episodeId,
			completed,
		});
		entry = entry;
	}

	/// episode might be null
	function ctxMenu(e, seasonIdx: number, episodeId?: string) {
		e.preventDefault();
		e.stopPropagation();

		contextMenu.open(
			e,
			[
				{ id: 'reset', text: 'Nonid gluegt' },
				{ id: 'setCompleted', text: 'Scho gluegt' },
			],
			id => {
				if (id === 'reset') setCompleted(seasonIdx, episodeId, false);
				else if (id === 'setCompleted')
					setCompleted(seasonIdx, episodeId, true);
				// else if (id === '')
			},
		);
	}

	function seasonTitle(season: Season): string {
		return (
			'Season ' +
			padZero(season.season) +
			(season.name ? ' ' + season.name : '')
		);
	}

	function episodeTitle(episode: Episode): string {
		return padZero(episode.episode) + ' - ' + episode.name;
	}
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->
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
				{#each entry.seasons as season, idx}
					<div class="season">
						<span
							class="entry"
							style="--progress: {season.totalPercent() * 100}%"
							on:click={() => {
								openSeason = openSeason === idx ? null : idx;
							}}
							on:contextmenu={e => ctxMenu(e, idx, null)}
						>
							<span>{seasonTitle(season)}</span>
						</span>

						{#if openSeason === idx}
							<div
								class="season-list"
								transition:svelteSlide={{ duration: 200 }}
							>
								{#each season.episodes as episode}
									<span
										class="entry"
										style="--progress: {episode.percent() *
											100}%"
										on:click={() =>
											onSelectEpisode(episode.id)}
										on:contextmenu={e =>
											ctxMenu(e, idx, episode.id)}
									>
										<span>{episodeTitle(episode)}</span>
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
