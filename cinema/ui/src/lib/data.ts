import { padZero, sortToHigher, sortToLower } from 'chuchi-utils';
import {
	entries as entriesApi,
	MIN_PERCENT,
	MAX_PERCENT,
	Entry as ApiEntry,
	ProgressStream,
	Movie,
	Series,
	Season,
	Episode,
} from './api.js';
import DateTime from 'chuchi-legacy/time/DateTime';
import { searchScore } from 'chuchi-utils/search';

class Entries {
	list: Entry[];

	/// entries: [Entry]
	constructor(entries: ApiEntry[]) {
		this.list = entries.map(e => new Entry(e));
	}

	get(id: string) {
		return this.list.find(e => e.id() == id) ?? null;
	}

	toDashboard(): DashboardEntries {
		return new DashboardEntries(this);
	}

	search(val: string) {
		const filter = new SearchFilter(val);

		return this.list
			.map(e => {
				const score = filter.match(e);
				return [score, e] as [number, Entry];
			})
			.filter(([score, e]) => score !== 0)
			.sort((a, b) => {
				return filter.sort(a, b);
			})
			.map(([score, e]) => e);
	}
}

class SearchFilter {
	private raw: string;
	private text: string;

	kind: 'Movie' | 'Series' | null;
	order: { field: 'Year' | 'Updated'; order: 'Asc' | 'Desc' } | null;
	// we need to parse filter and ordering
	// kind:movie
	// k:m

	// order:year
	// o:y
	// order:year:asc
	constructor(s: string) {
		this.raw = s;

		// Movie|Series|null
		this.kind = null;
		// { field: Year|Updated, order: Asc|Desc }|null
		this.order = null;

		this.text = '';

		this._parse(s);
	}

	match(e: Entry): number {
		// lets first apply the filter
		if (this.kind && e.kind !== this.kind) return 0;

		if (!this.text) return 1;

		return e.match(this.text);
	}

	sort([aScore, ae]: [number, Entry], [bScore, be]: [number, Entry]): number {
		if (!this.order) return sortToHigher(aScore, bScore);

		let prevOrder = 0;

		const { field, order } = this.order;
		let av = null;
		let bv = null;
		if (field === 'Year') {
			if (ae.inner.kind === 'Movie' && be.inner.kind === 'Movie') {
				av = ae.inner.data.year();
				bv = be.inner.data.year();
			}
		} else if (field === 'Updated') {
			av = ae.updatedOn().time;
			bv = be.updatedOn().time;
		}

		if (av && bv) {
			if (order === 'Asc') prevOrder = sortToHigher(av, bv);
			else prevOrder = sortToLower(av, bv);
		}

		if (prevOrder !== 0) return prevOrder;

		return sortToHigher(aScore, bScore);
	}

	private _parse(s: string) {
		const text = [];

		for (const word of s.split(' ')) {
			const filter = word.toLowerCase().split(':');
			if (filter.length === 1 || filter.some(f => !f.length)) {
				text.push(word);
				continue;
			}

			const k = filter[0];
			const v = filter[1];
			const add = filter[2] ?? null;
			let order: 'Asc' | 'Desc';

			switch (k) {
				case 'k':
				case 'kind':
					if (v.startsWith('m')) this.kind = 'Movie';
					else if (v.startsWith('s')) this.kind = 'Series';
					console.log('invalid value for kind', v);
					break;

				case 'o':
				case 'order':
					if (add && add.startsWith('a')) order = 'Asc';
					else order = 'Desc';

					if (v.startsWith('y'))
						this.order = { field: 'Year', order };
					else if (v.startsWith('u'))
						this.order = { field: 'Updated', order };
					break;
			}
		}

		this.text = text.join(' ');
	}
}

const IS_NEWEST = 2 * 7 * 24 * 60 * 60 * 1000;

export class DashboardEntries {
	inner: Entries;
	lastWatched: Entry | null;
	watchLater: Entry[];
	newest: Entry[];
	movies: Entry[];
	series: Entry[];

	// entries: Entries
	constructor(entries: Entries) {
		this.inner = entries;

		this.lastWatched = null;
		this.watchLater = [];
		this.newest = [];
		this.movies = [];
		this.series = [];

		this._splitEntries(this.inner.list);
		this._sort();
	}

	_replaceLastWatched(entry: Entry) {
		let prev = this.lastWatched;
		this.lastWatched = entry;
		return prev;
	}

	_splitEntries(list: Entry[]) {
		let lastWatchedTime = null;
		list.forEach(entry => {
			let updatedOn = entry.progressUpdatedOn();
			// check if the entry might be the lastWatched
			if (
				updatedOn &&
				(!lastWatchedTime || lastWatchedTime.time < updatedOn.time)
			) {
				lastWatchedTime = updatedOn;
				entry = this._replaceLastWatched(entry);
				if (!entry) return;
			}

			// categories to other categories
			let percent = entry.percent();

			// watchLater
			if (percent > MIN_PERCENT && percent < MAX_PERCENT) {
				this.watchLater.push(entry);
				return;
			}

			// newest
			let sinceUpdated = Date.now() - entry.updatedOn().time;
			if (sinceUpdated < IS_NEWEST) {
				this.newest.push(entry);
				return;
			}

			switch (entry.kind) {
				case 'Movie':
					this.movies.push(entry);
					break;
				case 'Series':
					this.series.push(entry);
					break;
			}
		});
	}

	_sort() {
		this.watchLater = this.watchLater.sort((a, b) => {
			const aTime = a.progressUpdatedOn()?.time ?? 0;
			const bTime = b.progressUpdatedOn()?.time ?? 0;
			return sortToLower(aTime, bTime);
		});
		this.newest = this.newest.sort((a, b) => {
			return sortToLower(a.updatedOn().time, b.updatedOn().time);
		});
		this.movies = this.movies.sort((a, b) => {
			return sortToHigher(a.title(), b.title());
		});
		this.series = this.series.sort((a, b) => {
			return sortToHigher(a.title(), b.title());
		});
	}
}

let entries = null;

export async function loadEntries(token: string) {
	if (entries) return new Entries(entries.list);

	entries = await entriesApi(token);
	return new Entries(entries.list);
}

export async function loadEntry(id: string, token: string) {
	const entrs = await loadEntries(token);
	return entrs.get(id);
}

type EntryInner =
	| { kind: 'Movie'; data: MovieEntry }
	| { kind: 'Series'; data: SeriesEntry };

export class Entry {
	/// entry: Entry
	inner: EntryInner;

	constructor(entry: ApiEntry) {
		switch (entry.inner.kind) {
			case 'Movie':
				this.inner = {
					kind: 'Movie',
					data: new MovieEntry(entry.inner.data),
				};
				break;
			case 'Series':
				this.inner = {
					kind: 'Series',
					data: new SeriesEntry(entry.inner.data),
				};
				break;
		}
	}

	get kind(): string {
		return this.inner.kind;
	}

	get data(): MovieEntry | SeriesEntry {
		return this.inner.data;
	}

	id(): string {
		return this.data.id();
	}

	title(): string {
		return this.data.title();
	}

	currentShortTitle(): string {
		return this.data.currentShortTitle();
	}

	currentTitle(): string {
		return this.data.currentTitle();
	}

	match(val: string): number {
		return this.data.match(val);
	}

	poster(): string {
		return this.data.inner.poster();
	}

	fullPoster(): string {
		return this.data.inner.fullPoster();
	}

	src(): string {
		return this.data.src();
	}

	// when was the movie or series last updated
	updatedOn(): DateTime {
		return this.data.inner.getUpdatedOn();
	}

	// might return null
	progressUpdatedOn(): DateTime | null {
		return this.data.inner.progressUpdatedOn();
	}

	percent(): number {
		return this.data.percent();
	}

	currentPercent(): number {
		return this.data.currentPercent();
	}

	position(): number {
		return this.data.position();
	}

	creditsPercent(totalLen: number): number {
		return this.data.creditsPercent(totalLen);
	}

	creditsTime(totalLen: number): number {
		return this.data.creditsTime(totalLen);
	}

	// this returns the progress percentage without the last few seconds
	// which can be skipped
	calcPercent(position: number, totalLen: number): number {
		const perc = this.data.calcPercent(position, totalLen);
		return Math.max(Math.min(perc, 1), 0);
	}

	setProgress(percent: number, position: number) {
		this.data.setProgress(percent, position);
	}

	sendProgress(stream: ProgressStream) {
		this.data.sendProgress(stream);
	}
}

export class MovieEntry {
	inner: Movie;

	constructor(movie: Movie) {
		this.inner = movie;
	}

	id(): string {
		return this.inner.id;
	}

	title(): string {
		return this.inner.title();
	}

	year(): number {
		return this.inner.year;
	}

	currentShortTitle(): string {
		return this.title();
	}

	currentTitle(): string {
		return this.title();
	}

	match(val: string): number {
		return searchScore(val, this.title());
	}

	src(): string {
		return this.inner.src();
	}

	percent(): number {
		return this.inner.percent();
	}

	currentPercent(): number {
		return this.percent();
	}

	position(): number {
		return this.inner.position();
	}

	creditsPercent(totalLen: number): number {
		return this.inner.creditsDuration(totalLen) / totalLen;
	}

	creditsTime(totalLen: number): number {
		return totalLen - this.inner.creditsDuration(totalLen);
	}

	calcPercent(position: number, totalLen: number): number {
		return position / this.creditsTime(totalLen);
	}

	setProgress(percent: number, position: number) {
		this.inner.setProgress(percent, position);
	}

	sendProgress(stream: ProgressStream) {
		this.inner.sendProgress(stream);
	}
}

export class SeriesEntry {
	inner: Series;
	cSeason: number;
	cEpisode: number;

	// series: Series
	constructor(series: Series) {
		this.inner = series;
		this.cSeason = 0;
		this.cEpisode = 0;

		// we need to calculate the progress
		this.calcCurrentEpisode();
	}

	calcCurrentEpisode() {
		const seasons = this.inner.seasons;
		for (let se = 0; se < seasons.length; se++) {
			const ep = seasons[se].episodes.findIndex(e => !e.isCompleted());
			if (ep !== -1) {
				this.cSeason = se;
				this.cEpisode = ep;
				break;
			}
		}
	}

	seasons(): Season[] {
		return this.inner.seasons;
	}

	season(): Season {
		return this.inner.seasons[this.cSeason];
	}

	episode(): Episode {
		return this.inner.seasons[this.cSeason].episodes[this.cEpisode];
	}

	setEpisode(seasonIdx: number, episodeIdx: number) {
		this.cSeason = seasonIdx;
		this.cEpisode = episodeIdx;
	}

	/// returns null or [season, episode]
	nextEpisode(): [number, number] | null {
		// check if we can move to next episode (not season)
		if (this.cEpisode + 1 < this.season().episodes.length)
			return [this.cSeason, this.cEpisode + 1];

		const nSeason = this.cSeason + 1;
		if (this.seasons()[nSeason]?.episodes.length > 0)
			return [this.cSeason + 1, 0];

		return null;
	}

	id(): string {
		return this.inner.id;
	}

	title(): string {
		return this.inner.name;
	}

	currentShortTitle(): string {
		const ep = this.episode();
		const season = padZero(this.cSeason + 1);
		const episode = padZero(this.cEpisode + 1);
		return `S${season}E${episode} ${ep.name}`;
	}

	currentTitle(): string {
		return `${this.title()} ${this.currentShortTitle()}`;
	}

	match(val: string): number {
		return searchScore(val, this.title());
	}

	// menu
	// video src
	// position

	src(): string {
		return this.inner.src(this.cSeason, this.cEpisode);
	}

	percent(): number {
		const l = this.inner.seasons.length;
		return this.inner.seasons.reduce((t, c) => c.totalPercent() / l + t, 0);
	}

	currentPercent(): number {
		return this.inner.percent(this.cSeason, this.cEpisode);
	}

	position(): number {
		return this.inner.position(this.cSeason, this.cEpisode);
	}

	creditsPercent(totalLen: number): number {
		const dur = this.inner.creditsDuration(
			this.cSeason,
			this.cEpisode,
			totalLen,
		);
		return dur / totalLen;
	}

	creditsTime(totalLen: number): number {
		return (
			totalLen -
			this.inner.creditsDuration(this.cSeason, this.cEpisode, totalLen)
		);
	}

	calcPercent(position: number, totalLen: number): number {
		// 2 minutes
		return position / this.creditsTime(totalLen);
	}

	setProgress(percent: number, position: number) {
		this.inner.setProgress(this.cSeason, this.cEpisode, percent, position);
	}

	sendProgress(stream: ProgressStream) {
		this.inner.sendProgress(stream, this.cSeason, this.cEpisode);
	}
}
