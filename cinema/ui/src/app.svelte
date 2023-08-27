<script>
	import { tick } from 'svelte';
	import { timeout } from 'fire/util.js';
	import { stream } from './lib/api.js';
	import { loadEntries } from './lib/data.js';
	import * as core from 'core-lib';
	const { router } = core.router;
	const { session } = core.user;
	const { open: openCtx } = core.contextmenu;
	import Cover from './ui/cover.svelte';
	import BackBtn from 'core-lib-ui/back-btn';
	import Search from 'core-lib-ui/search';

	let searchValue = '';
	export { searchValue as search };
	let initialSearchValue = searchValue;

	let contEl = null;

	let entries = null;
	let searchEntries = null;
	const groups = [
		['watchLater', false, 'Später aluege'],
		['newest', true, 'Neu hinzuegfüegt'],
		['series', true, 'Serien'],
		['movies', true, 'Filme']
	];

	async function load() {
		const entrs = await loadEntries(session.getValid().token);
		entries = entrs.toDashboard();

		searchChange(searchValue);
	}
	load();

	/* Events */
	let tooltipOpen = false;
	function onTooltipClick(e) {
		tooltipOpen = !tooltipOpen;
	}

	async function clickViewMore(e, group) {
		e.preventDefault();
		e.stopPropagation();

		if (group === 'newest') {
			searchValue = 'order:update';
		} else {
			searchValue = 'kind:' + group;
		}

		// because we wait for the next tick we make sure
		// that the history state was pushed before scrolling to the top
		// meaning if you go back the correct scroll position will be shown
		await tick();

		window.scrollTo({
			top: 0
		});
	}

	function ctxMenu(e, entry) {
		console.log('ctx', entry);

		if (entry.kind !== 'Movie')
			return;

		e.preventDefault();
		e.stopPropagation();

		// todo to make this work we need to add a new api
		// to set progress

		// openCtx(e,
		// 	[
		// 		{ id: 'reset', text: 'Nonid gluegt' },
		// 		{ id: 'setCompleted', text: 'Scho gluegt' }
		// 	],
		// 	id => {
		// 		console.log('id', id);
		// 	}
		// );
	}

	function searchChange(val) {
		if (!entries)
			return;

		const nReq = router.currentReq().clone();
		nReq.search.set('search', val);

		// if there we didn't do any searching already
		// push a new page with the search
		// else just replace the search parameter
		if (!initialSearchValue && val) {
			router.pushReq(nReq);
			initialSearchValue = val;
		} else if (initialSearchValue) {
			router.replaceReq(nReq);
		}

		if (val) {
			searchEntries = entries.inner.search(val);
		} else {
			searchEntries = null;
		}
	}
	$: searchChange(searchValue);
	
</script>

<div id="cinema" bind:this={contEl}>
	<header>
		<BackBtn href="/" onlyHref={true} />

		<div class="center">
			<Search bind:value={searchValue}>
				<button class="tooltip" on:click={onTooltipClick}>i</button>
				<div class="tooltip-box" class:open={tooltipOpen}>
					<p>
						<strong>Filter nach typ:</strong><br>
						Nach film: <i>kind:movie</i> oder <i>k:m</i><br>
						Nach Serie: <i>kind:series</i> oder <i>k:s</i>
					</p>
					<p>
						<strong>Sortieren:</strong><br>
						Nach Jahr: <i>order:year</i> oder <i>o:y</i><br>
						Nach hochladedatum: <i>order:update</i> oder <i>o:u</i><br>
						Richtung ändern: <i>order:year:asc</i>
					</p>
				</div>
			</Search>
		</div>
	</header>

	{#if searchEntries !== null}
		<div class="hero-placeholder"></div>
		<section>
			<!-- <h2>{groupTitle}</h2> -->

			<div class="list">
				{#each searchEntries as entry}
					<a href="/cinema/watch/{entry.id()}" class="entry">
						<Cover
							src={entry.poster()}
							alt={entry.title()}
							percent={entry.percent()}
						/>
						<div class="over">
							<h4>{entry.title()}</h4>
						</div>
					</a>
				{/each}
			</div>
		</section>
	{:else if entries}
		{@const lastWatched = entries.lastWatched}
		{#if lastWatched}
			<div class="hero bg-image">
				<div class="shader"></div>

				<div class="hero-entry">
					<Cover
						src={lastWatched.fullPoster()}
						alt={lastWatched.title()}
						percent={lastWatched.percent()}
						on:click={() => router.open(`/cinema/watch/${lastWatched.id()}`)}
						on:contextmenu={e => ctxMenu(e, lastWatched)}
					/>
					<div class="info">
						<h1>{lastWatched.title()}</h1>
						<!-- <p>{lastWatched.percent() * 100}% gluegt</p> -->
						<a href="/cinema/watch/{lastWatched.id()}" class="action-btn">
							Abspile
						</a>
					</div>
				</div>
			</div>
		{:else}
			<div class="hero-placeholder"></div>
		{/if}

		{#each groups as [group, filterAble, groupTitle]}
			{@const list = filterAble ? entries[group].slice(0, 6) : entries[group]}
			{#if list.length > 0}
				<section>
					<h2>{groupTitle}</h2>

					<div class="list" class:filter-able={filterAble}>
						{#each list as entry}
							<a href="/cinema/watch/{entry.id()}" class="entry" on:contextmenu={e => ctxMenu(e, entry)}>
								<Cover
									src={entry.poster()}
									alt={entry.title()}
									percent={entry.percent()}
								/>
								<div class="over">
									<h4>{entry.title()}</h4>
								</div>
							</a>
						{/each}
						{#if filterAble}
							<a
								href="/cinema/?search=kind%3A{group}"
								class="viewmore"
								on:click={e => clickViewMore(e, group)}
							>
								<h4>Meh</h4>
							</a>
						{/if}
					</div>
				</section>
			{/if}
		{/each}
	{/if}
</div>

<style lang="scss">
	#cinema {
		min-height: 100vh;
	}

	header {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		display: grid;
		grid-template-columns: 40px 1fr 40px;
		grid-gap: 20px;
		justify-content: space-between;
		margin-top: 20px;
		z-index: 2;
	}

	.center {
		display: flex;
		justify-content: center;
	}

	.tooltip {
		position: absolute;
		top: calc(50% - 12px);
		right: 10px;
		width: 24px;
		height: 24px;
		background-color: transparent;
		outline: none;
		border: 1px solid var(--dark-border-color);
		border-radius: 50%;
		font-size: 16px;
		cursor: pointer;
		font-family: serif;
	}

	.tooltip-box {
		position: absolute;
		display: none;
		top: calc(100% + 5px);
		width: 100%;
		background-color: var(--gray);
		border-radius: 8px;
		border: 1px solid var(--dark-border-color);
		padding: 9px 15px;

		&.open {
			display: block;
		}

		p {
			margin-bottom: 10px;
			color: #a3a3a3;
		}

		i, strong {
			color: white;
			font-style: normal;
		}
	}

	.hero {
		position: relative;
		/* should be sinced with the grid-template-column for hero-entry */
		height: calc(20vw * 1.5873);
		background-attachment: fixed;
		margin-bottom: 180px;
	}

	.shader {
		position: absolute;
		left: 0;
		bottom: 0;
		width: 100%;
		height: 300px;
		background: linear-gradient(0deg, var(--dark-gray), transparent);
	}

	.hero-entry {
		position: absolute;
		display: grid;
		left: 120px;
		bottom: -100px;
		width: calc((100% - 240px) * 0.8);
		grid-template-columns: 20vw 1fr;
		grid-gap: 20px;
		align-items: center;
		z-index: 1;
	}

	.hero-entry :global(.cover) {
		cursor: pointer;
	}

	.info p {
		color: #979797;
	}

	.action-btn {
		margin-top: 10px;
	}

	.hero-placeholder {
		height: 120px;
	}

	section {
		padding: 0 120px;
		margin-bottom: 50px;
	}

	h2 {
		margin-bottom: 15px;
	}

	.list {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		grid-gap: 5px;
	}

	.entry {
		position: relative;
		display: block;
	}

	.over {
		position: absolute;
		display: flex;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		padding: 10px;
		align-items: center;
		justify-content: center;
		background-color: rgba(0, 0, 0, .7);
		text-align: center;
		opacity: 0;

		transition: opacity ease .2s;
	}

	.entry:hover .over {
		opacity: 1;
	}

	.viewmore {
		aspect-ratio: 0.63;
		display: flex;
		padding: 10px;
		align-items: center;
		justify-content: center;
		background-color: rgb(172 34 62 / 60%);
		text-decoration: none;
		text-align: center;

		transition: background-color ease .2s;

		&:hover {
			background-color: var(--red);
		}
	}

	h4 {
		font-size: 20px;
		font-weight: 600;
	}


	@media (max-width: 1500px) {
		section {
			margin-bottom: 40px;
		}

		h2 {
			margin-bottom: 10px;
		}

		.list {
			grid-template-columns: repeat(6, 1fr);
		}

		.filter-able .entry:nth-child(6) {
			display: none;
		}
	}

	@media (max-width: 1300px) {
		section {
			padding: 0 60px;
		}

		.hero-entry {
			left: 60px;
			width: calc(100% - 120px);
		}

		.list {
			grid-template-columns: repeat(5, 1fr);
		}

		.filter-able .entry:nth-child(5) {
			display: none;
		}
	}

	@media (max-width: 1000px) {
		.hero {
			height: calc(25vw * 1.5873);
		}

		.hero-entry {
			grid-template-columns: 25vw 1fr;
		}

		.list {
			grid-template-columns: repeat(4, 1fr);
		}

		.filter-able .entry:nth-child(4) {
			display: none;
		}
	}

	@media (max-width: 900px) {
		section {
			padding: 0 30px;
		}

		.hero {
			height: calc(20vw * 1.5873);
			margin-bottom: 140px;
		}

		.hero-entry {
			left: 30px;
			width: calc(100% - 60px);
			grid-template-columns: 20vw 1fr;
		}
	}

	@media (max-width: 500px) {
		section {
			padding: 0 20px;
			margin-bottom: 30px;
		}

		.hero {
			height: calc(30vw * 1.5873);
			margin-bottom: 140px;
		}

		.hero-entry {
			left: 20px;
			width: calc(100% - 40px);
			grid-template-columns: 30vw 1fr;
		}

		.list {
			grid-template-columns: repeat(3, 1fr);
		}

		.filter-able .entry:nth-child(3) {
			display: none;
		}
	}
</style>