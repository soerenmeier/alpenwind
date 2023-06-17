<script>
	import { afterUpdate } from 'svelte';
	import { timeout } from 'fire/util.js';
	import { stream } from './lib/api.js';
	import { loadEntries } from './lib/data.js';
	import * as core from 'core-lib';
	const { router } = core.router;
	const { session } = core.user;
	import Cover from './ui/cover.svelte';
	import BackBtn from 'core-lib-ui/back-btn';
	import Search from 'core-lib-ui/search';

	let contEl = null;

	let entries = null;
	let searchEntries = null;
	const groups = [
		['watchLater', 'Später aluege'],
		['newest', 'Neu hinzuegfüegt'],
		['series', 'Serien'],
		['movies', 'Filme']
	];

	let scrollSet = 0;// 0: no, 1: ready, 2: set

	let searchValue = router.currentState().searchValue ?? '';
	let scrollTop = router.currentState().scrollTop ?? 0;

	async function load() {
		const entrs = await loadEntries(session.getValid().token);
		entries = entrs.toDashboard();

		searchChange(searchValue);

		scrollSet = 1;
	}
	load();

	/* Events */
	function searchChange(val) {
		if (!entries)
			return;

		const s = router.currentState();
		s.searchValue = searchValue;
		router.replaceState(s);

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
			<Search bind:value={searchValue} />
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
						percent={lastWatched.percent()}
						on:click={() => router.open(`/cinema/watch/${lastWatched.id()}`)}
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

		{#each groups as [group, groupTitle]}
			{@const list = entries[group]}
			{#if list.length > 0}
				<section>
					<h2>{groupTitle}</h2>

					<div class="list">
						{#each list as entry}
							<a href="/cinema/watch/{entry.id()}" class="entry">
								<Cover
									src={entry.poster()}
									percent={entry.percent()}
								/>
								<div class="over">
									<h4>{entry.title()}</h4>
								</div>
							</a>
						{/each}
					</div>
				</section>
			{/if}
		{/each}
	{/if}
</div>

<style>
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
	}

	@media (max-width: 1300px) {
		.list {
			grid-template-columns: repeat(5, 1fr);
		}

		section {
			padding: 0 60px;
		}

		.hero-entry {
			left: 60px;
			width: calc(100% - 120px);
		}
	}

	@media (max-width: 1000px) {
		.list {
			grid-template-columns: repeat(4, 1fr);
		}

		.hero {
			height: calc(25vw * 1.5873);
		}

		.hero-entry {
			grid-template-columns: 25vw 1fr;
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
		.list {
			grid-template-columns: repeat(3, 1fr);
		}

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
	}
</style>