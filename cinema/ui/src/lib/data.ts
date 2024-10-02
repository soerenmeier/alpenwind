import { padZero, range, sortToHigher, sortToLower } from 'chuchi-utils';
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
	EntriesResp,
	ProgressId,
} from './api.js';
import DateTime from 'chuchi-legacy/time/DateTime';
import { searchScore } from 'chuchi-utils/search';

class Entries {
	list: Entry[];

	/// entries: [Entry]
	constructor(entries: ApiEntry[]) {
		this.list = entries.map(e => newEntry(e));
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
		if (this.kind && e.kind() !== this.kind) return 0;

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
			av = ae.year();
			bv = be.year();
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
					else console.log('invalid value for kind', v);
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

	getByGroup(group: string): Entry[] {
		return this[group];
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

			switch (entry.kind()) {
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

let entries: EntriesResp | null = null;

export async function loadEntries(token: string) {
	if (entries) return new Entries(entries.list);

	entries = await entriesApi(token);
	return new Entries(entries.list);
}

export async function loadEntry(id: string, token: string) {
	const entrs = await loadEntries(token);
	return entrs.get(id);
}

export type PosterResolution = 'small' | 'full';

export interface Entry {
	id(): string;

	kind(): 'Movie' | 'Series';

	year(): number | null;

	updatedOn(): DateTime;

	title(): string;

	progressUpdatedOn(): DateTime | null;

	percent(): number;

	match(s: string): number;

	poster(resolution?: PosterResolution): string;

	progressId(): ProgressId;

	// the source of the current active video
	activeSrc(): string;

	activePercent(): number;

	creditsPercent(duration: number): number;

	setProgress(percent: number): void;
}

function newEntry(entry: ApiEntry): Entry {
	switch (entry.kind) {
		case 'Movie':
			return new MovieEntry(entry);
		case 'Series':
			return new SeriesEntry(entry);
	}
}

export class MovieEntry implements Entry {
	inner: ApiEntry;
	data: Movie;

	constructor(entry: ApiEntry) {
		this.inner = entry;
		this.data = entry.data.data as Movie;
	}

	id(): string {
		return this.inner.id;
	}

	kind(): 'Movie' {
		return 'Movie';
	}

	year(): number | null {
		return this.data.year;
	}

	updatedOn(): DateTime {
		return this.inner.updatedOn;
	}

	title(): string {
		return this.inner.name + ' ' + this.data.year;
	}

	progressUpdatedOn(): DateTime | null {
		return this.data.progress?.updatedOn ?? null;
	}

	percent(): number {
		return this.data.progress?.percent ?? 0;
	}

	match(val: string): number {
		return searchScore(val, this.title());
	}

	poster(resolution: PosterResolution = 'small'): string {
		if (resolution === 'full') {
			return this.data.fullPosterUrl(this.inner.name);
		}
		return this.data.posterUrl(this.inner.name);
	}

	progressId(): ProgressId {
		return {
			kind: 'Movie',
			id: this.id(),
		};
	}

	activeSrc(): string {
		return this.data.videoSrc(this.inner.name);
	}

	activePercent(): number {
		return this.data.progress?.percent ?? 0;
	}

	creditsPercent(duration: number): number {
		return this.data.creditsDuration(duration) / duration;
	}

	private creditsTime(totalLen: number): number {
		return totalLen - this.data.creditsDuration(totalLen);
	}

	setProgress(percent: number) {
		this.data.setProgress(percent);
	}
}

export class SeriesEntry implements Entry {
	inner: ApiEntry;
	data: Series;
	seasons: Season[];
	// not the number but the index
	cSeason: number;
	// not the number but the index
	cEpisode: number;

	// series: Series
	constructor(entry: ApiEntry) {
		this.inner = entry;
		this.data = entry.data.data as Series;
		this.seasons = this.data.seasons;
		this.cSeason = 0;
		this.cEpisode = 0;

		// we need to calculate the progress
		this.calcCurrentEpisode();
	}

	private calcCurrentEpisode() {
		this.cSeason = this.seasons.findIndex(s => {
			const ep = s.episodes.findIndex(e => !e.progress?.isCompleted());
			if (ep > -1) {
				this.cEpisode = ep;
				return true;
			}
		});
		this.cSeason = Math.max(this.cSeason, 0);
	}

	season(): Season {
		return this.seasons[this.cSeason];
	}

	episode(): Episode {
		return this.season().episodes[this.cEpisode];
	}

	id(): string {
		return this.inner.id;
	}

	kind(): 'Series' {
		return 'Series';
	}

	year(): number | null {
		return null;
	}

	updatedOn(): DateTime {
		return this.inner.updatedOn;
	}

	title(): string {
		return this.inner.name;
	}

	progressUpdatedOn(): DateTime | null {
		return this.episode().progress?.updatedOn ?? null;
	}

	percent(): number {
		return this.episode().percent();
	}

	match(val: string): number {
		return searchScore(val, this.title());
	}

	poster(resolution: PosterResolution = 'small'): string {
		if (resolution === 'full') {
			return this.data.fullPosterUrl(this.inner.name);
		}
		return this.data.posterUrl(this.inner.name);
	}

	progressId(): ProgressId {
		return {
			kind: 'Episode',
			id: this.episode().id,
		};
	}

	activeSrc(): string {
		return this.data.videoSrc(this.inner.name, this.cSeason, this.cEpisode);
	}

	activePercent(): number {
		return this.episode().progress?.percent ?? 0;
	}

	// todo do we wan't credits percent and not duration?
	creditsPercent(duration: number): number {
		return this.episode().creditsDuration(duration) / duration;
	}

	calcPercent(position: number, totalLen: number): number {
		// 2 minutes
		const perc = position / this.creditsTime(totalLen);
		return Math.max(Math.min(perc, 1), 0);
	}

	private creditsTime(totalLen: number): number {
		return totalLen - this.episode().creditsDuration(totalLen);
	}

	setProgress(percent: number) {
		this.episode().setProgress(percent);
	}

	setProgressOnEpisode(
		seasonIdx: number,
		episodeId: string | null,
		percent: number,
	): [ProgressId, number][] {
		const season = this.seasons[seasonIdx];

		let episodes: number[] = [];
		if (typeof episodeId === 'string') {
			episodes = [season.episodes.findIndex(e => e.id === episodeId)];
		} else {
			episodes = range(0, season.episodes.length);
		}

		return episodes.map(idx => {
			const episode = season.episodes[idx];
			episode.setProgress(percent);

			return [{ kind: 'Episode', id: episode.id }, percent];
		});
	}

	setEpisode(episodeId: string) {
		const found = this.seasons
			.map((s, i) => {
				const idx = s.episodes.findIndex(e => e.id === episodeId);
				return idx > -1 ? [i, idx] : null;
			})
			.find(e => e !== null);

		if (!found) throw new Error('could not find episode ' + episodeId);

		this.cSeason = found[0];
		this.cEpisode = found[1];
	}

	// returns true if the episode was updated
	nextEpisode(): boolean {
		// check if we can move to next episode (not season)
		if (this.cEpisode + 1 < this.season().episodes.length) {
			this.cEpisode++;
			return true;
		}

		const nSeason = this.cSeason + 1;
		if (this.seasons[nSeason]?.episodes.length > 0) {
			this.cSeason++;
			this.cEpisode = 0;
			return true;
		}

		return false;
	}
}
