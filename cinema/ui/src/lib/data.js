import { padZero, sortToHigher, sortToLower, match } from 'fire/util.js';
import { entries as entriesApi, MIN_PERCENT, MAX_PERCENT } from './api.js';

class Entries {
	/// entries: [Entry]
	constructor(entries) {
		this.list = entries.map(e => new Entry(e));
	}

	get(id) {
		return this.list.find(e => e.id() == id) ?? null;
	}

	toDashboard() {
		return new DashboardEntries(this);
	}

	search(val) {
		return this.list.map(e => {
			const score = e.match(val);
			return [score, e];
		})
			.filter(([score, e]) => score !== 0)
			.sort(([aScore, ae], [bScore, be]) => sortToHigher(aScore, bScore))
			.map(([score, e]) => e);
	}
}

const IS_NEWEST = 2 * 7 * 24 * 60 * 60 * 1000;

export class DashboardEntries {
	// entries: Entries
	constructor(entries) {
		this.inner = entries;

		this.lastWatched = null;
		this.watchLater = [];
		this.newest = [];
		this.movies = [];
		this.series = [];

		this._splitEntries(this.inner.list);
		this._sort();
	}

	_replaceLastWatched(entry) {
		let prev = this.lastWatched;
		this.lastWatched = entry;
		return prev;
	}

	_splitEntries(list) {
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
				if (!entry)
					return;
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

export async function loadEntries(token) {
	if (entries)
		return new Entries(entries.list);

	entries = await entriesApi(token);
	return new Entries(entries.list);
}

export async function loadEntry(id, token) {
	const entrs = await loadEntries(token);
	return entrs.get(id);
}

export class Entry {
	/// entry: Entry
	constructor(entry) {
		this.kind = entry.kind;
		switch (entry.kind) {
			case 'Movie':
				this.data = new MovieEntry(entry.data);
				break;
			case 'Series':
				this.data = new SeriesEntry(entry.data);
				break;
		}
	}

	id() {
		return this.data.id();
	}

	title() {
		return this.data.title();
	}

	currentShortTitle() {
		return this.data.currentShortTitle();
	}

	currentTitle() {
		return this.data.currentTitle();
	}

	match(val) {
		return this.data.match(val);
	}

	poster() {
		return this.data.inner.poster();
	}

	fullPoster() {
		return this.data.inner.fullPoster();
	}

	src() {
		return this.data.src();
	}

	// when was the movie or series last updated
	updatedOn() {
		return this.data.inner.getUpdatedOn();
	}

	// might return null
	progressUpdatedOn() {
		return this.data.inner.progressUpdatedOn();
	}

	percent() {
		return this.data.percent();
	}

	currentPercent() {
		return this.data.currentPercent();
	}

	position() {
		return this.data.position();
	}

	creditsPercent(totalLen) {
		return this.data.creditsPercent(totalLen);
	}

	creditsTime(totalLen) {
		return this.data.creditsTime(totalLen);
	}

	// this returns the progress percentage without the last few seconds
	// which can be skipped
	calcPercent(position, totalLen) {
		const perc = this.data.calcPercent(position, totalLen);
		return Math.max(Math.min(perc, 1), 0);
	}

	setProgress(percent, position) {
		this.data.setProgress(percent, position);
	}

	sendProgress(stream) {
		this.data.sendProgress(stream);
	}
}

export class MovieEntry {
	constructor(movie) {
		this.inner = movie;
	}

	id() {
		return this.inner.id;
	}

	title() {
		return this.inner.title();
	}

	currentShortTitle() {
		return this.title();
	}

	currentTitle() {
		return this.title();
	}

	match(val) {
		return match(val, this.title());
	}

	src() {
		return this.inner.src();
	}

	percent() {
		return this.inner.percent();
	}

	currentPercent() {
		return this.percent();
	}

	position() {
		return this.inner.position();
	}

	creditsPercent(totalLen) {
		return this.inner.creditsDuration(totalLen) / totalLen;
	}

	creditsTime(totalLen) {
		return totalLen - this.inner.creditsDuration(totalLen);
	}

	calcPercent(position, totalLen) {
		return position / this.creditsTime(totalLen);
	}

	setProgress(percent, position) {
		this.inner.setProgress(percent, position);
	}

	sendProgress(stream) {
		this.inner.sendProgress(stream);
	}
}

export class SeriesEntry {
	// series: Series
	constructor(series) {
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
				break
			}
		}
	}

	seasons() {
		return this.inner.seasons;
	}

	season() {
		return this.inner.seasons[this.cSeason];
	}

	episode() {
		return this.inner.seasons[this.cSeason].episodes[this.cEpisode];
	}

	setEpisode(seasonIdx, episodeIdx) {
		this.cSeason = seasonIdx;
		this.cEpisode = episodeIdx;
	}

	/// returns null or [season, episode]
	nextEpisode() {
		// check if we can move to next episode (not season)
		if (this.cEpisode + 1 < this.season().episodes.length)
			return [this.cSeason, this.cEpisode + 1];

		const nSeason = this.cSeason + 1;
		if (this.seasons()[nSeason]?.episodes.length > 0)
			return [this.cSeason + 1, 0];

		return null;
	}

	id() {
		return this.inner.id;
	}

	title() {
		return this.inner.name;
	}

	currentShortTitle() {
		const ep = this.episode();
		const season = padZero(this.cSeason + 1);
		const episode = padZero(this.cEpisode + 1);
		return `S${season}E${episode} ${ep.name}`;
	}

	currentTitle() {
		return `${this.title()} ${this.currentShortTitle()}`;
	}

	match(val) {
		return match(val, this.title());
	}

	// menu
	// video src
	// position

	src() {
		return this.inner.src(this.cSeason, this.cEpisode);
	}

	percent() {
		const l = this.inner.seasons.length;
		return this.inner.seasons.reduce((t, c) => c.totalPercent() / l + t, 0);
	}

	currentPercent() {
		return this.inner.percent(this.cSeason, this.cEpisode);
	}

	position() {
		return this.inner.position(this.cSeason, this.cEpisode);
	}

	creditsPercent(totalLen) {
		const dur = this.inner.creditsDuration(
			this.cSeason,
			this.cEpisode,
			totalLen
		);
		return dur / totalLen;
	}

	creditsTime(totalLen) {
		return totalLen - this.inner.creditsDuration(
			this.cSeason,
			this.cEpisode,
			totalLen
		);
	}

	calcPercent(position, totalLen) {
		// 2 minutes
		return position / this.creditsTime(totalLen);
	}

	setProgress(percent, position) {
		this.inner.setProgress(this.cSeason, this.cEpisode, percent, position);
	}

	sendProgress(stream) {
		this.inner.sendProgress(stream, this.cSeason, this.cEpisode);
	}
}