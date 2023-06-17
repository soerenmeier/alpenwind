import { padZero } from 'fire/util.js';
import Api from 'fire/api/api.js';
import Stream from 'fire/api/stream.js';
import Data from 'fire/data/data.js';
import DateTime from 'fire/data/datetime.js';
import { Option } from 'fire/data/parsetypes.js';
import * as core from 'core-lib';

export const MIN_PERCENT = 0.0001;
export const MAX_PERCENT = 0.9999;

const addr = core.api.serverAddr;
const api = new Api(addr + 'api/cinema/');
export let stream = new Stream(api, '/stream');

stream.onClose(() => {
	console.log('stream closed');
	alert('E Fähler isch passiert. Bitte lad d\'site neu');
});

export function assets(url) {
	return `${addr}assets/cinema/${url}`;
}

export function bgImg(url) {
	return `background-image: url("${assets(url)}")`;
}

export class Entry extends Data {
	constructor(d) {
		super({}, null);

		switch (Object.keys(d)[0]) {
			case 'Movie':
				this.kind = 'Movie';
				this.data = new Movie(d['Movie']);
				return;
			case 'Series':
				this.kind = 'Series';
				this.data = new Series(d['Series']);
				return;
		}
	}
}

export class Movie extends Data {
	constructor(d) {
		super({
			id: 'uid',
			name: 'str',
			year: 'optint',
			updatedOn: 'datetime',
			progress: new Option(Progress)
		}, d);
	}

	title() {
		return this.name + ' ' + this.year;
	}

	poster() {
		return assets(`posters/movies/${encodeURIComponent(this.title())}.jpg`);
	}

	fullPoster() {
		return assets(`full-posters/movies/${encodeURIComponent(this.title())}.jpg`);
	}

	src() {
		return assets(`movies/${encodeURIComponent(this.title())}.mp4`);
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(totalLen) {
		if (totalLen < 5 * 60)
			return 5;
		if (totalLen < 60 * 60)
			return 2 * 60;
		return 4 * 60;
	}

	getUpdatedOn() {
		return this.updatedOn;
	}

	progressUpdatedOn() {
		return this.progress?.updatedOn ?? null;
	}

	percent() {
		return this.progress?.percent ?? 0;
	}

	position() {
		return this.progress?.position ?? 0;
	}

	setProgress(percent, position) {
		this.progress = new Progress({
			percent, position,
			updatedOn: new DateTime
		});
	}

	sendProgress(stream) {
		const percent = this.progress?.percent ?? 0;
		const position = this.progress?.position ?? 0;
		stream.sendMovie(this.id, percent, position);
	}
}

export class Series extends Data {
	constructor(d) {
		super({
			id: 'uid',
			name: 'str',
			seasons: [Season]
		}, d);
	}

	poster() {
		return assets(`posters/series/${encodeURIComponent(this.name)}.jpg`);
	}

	fullPoster() {
		return assets(`full-posters/series/${encodeURIComponent(this.name)}.jpg`);
	}

	src(seasonIdx, episodeIdx) {
		const seas = this.seasons[seasonIdx];
		const season = encodeURIComponent(seas.folderName(seasonIdx));
		const ep = seas.episodes[episodeIdx];
		const episode = encodeURIComponent(ep.fileName(episodeIdx));
		return assets(`series/${this.name}/${season}/${episode}`);
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(seasonIdx, episodeIdx, totalLen) {
		return this.seasons[seasonIdx].episodes[episodeIdx].creditsDuration(
			totalLen);
	}

	getUpdatedOn() {
		let latest = null;
		this.seasons.forEach(season => {
			season.episodes.forEach(ep => {
				const updatedOn = ep.getUpdatedOn();
				if (updatedOn && !latest || latest.time < updatedOn.time)
					latest = updatedOn;
			});
		});

		if (!latest)
			throw new Error('should never be null');

		return latest;
	}

	progressUpdatedOn() {
		let latest = null;
		this.seasons.forEach(season => {
			season.episodes.forEach(ep => {
				const updatedOn = ep.progressUpdatedOn();
				if (updatedOn && (!latest || latest.time < updatedOn.time))
					latest = updatedOn;
			});
		});

		return latest;
	}

	percent(seasonIdx, episodeIdx) {
		return this.seasons[seasonIdx].episodes[episodeIdx].percent();
	}

	position(seasonIdx, episodeIdx) {
		return this.seasons[seasonIdx].episodes[episodeIdx].position();
	}

	setProgress(seasonIdx, episodeIdx, percent, position) {
		this.seasons[seasonIdx].episodes[episodeIdx]
			.setProgress(percent, position);
	}

	sendProgress(stream, seasonIdx, episodeIdx) {
		const ep = this.seasons[seasonIdx].episodes[episodeIdx];

		const percent = ep.progress?.percent ?? 0;
		const position = ep.progress?.position ?? 0;
		stream.sendSeries(this.id, seasonIdx, episodeIdx, percent, position);
	}
}

export class Progress extends Data {
	constructor(d) {
		// if (typeof d.updatedOn !== 'string')
		// 	debugger;
		super({
			// percent should be without the "deadzone"
			percent: 'float',
			position: 'float',
			updatedOn: DateTime
		}, d);
	}

	isCompleted() {
		return this.percent > MAX_PERCENT;
	}
}

export class Season extends Data {
	constructor(d) {
		super({
			name: 'optstr',
			episodes: [Episode]
		}, d);
	}

	title(idx) {
		let name = this.name ? ' ' + this.name : '';
		return 'Season ' + padZero(idx + 1) + name;
	}

	folderName(idx) {
		return this.title(idx);
	}

	totalPercent() {
		const l = this.episodes.length;
		return this.episodes.reduce((t, ep) => ep.percent() / l + t, 0);
	}
}

export class Episode extends Data {
	constructor(d) {
		super({
			name: 'str',
			updatedOn: 'datetime',
			progress: new Option(Progress)
		}, d);
	}

	isCompleted() {
		return this.progress && this.progress.isCompleted();
	}

	title(idx) {
		return `${padZero(idx + 1)} - ${this.name}`;
	}

	fileName(idx) {
		return `Episode ${padZero(idx + 1)} ${this.name}.mp4`;
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(totalLen) {
		if (totalLen < 5 * 60)
			return 5;
		return 20;
	}

	getUpdatedOn() {
		return this.updatedOn;
	}

	progressUpdatedOn() {
		return this.progress?.updatedOn ?? null;
	}

	percent() {
		return this.progress?.percent ?? 0;
	}

	position() {
		return this.progress?.position ?? 0;
	}

	setProgress(percent, position) {
		this.progress = new Progress({
			percent, position,
			updatedOn: new DateTime
		});
	}
}

export class EntriesResp extends Data {
	constructor(d) {
		super({
			list: [Entry]
		}, d);
	}
}

export async function entries(token) {
	const d = await api.request('POST', 'entries', null,
		{ 'auth-token': token }, { credentials: 'include' });
	return new EntriesResp(d);
}

export class ProgressStream {
	constructor(sender) {
		this.sender = sender;
	}

	async open(token) {
		if (!stream.isConnect())
			stream.connect();

		await this.sender.open({ token });
	}

	sendMovie(id, percent, position) {
		this.sender.send({
			'Movie': { id, percent, position }
		});
	}

	sendSeries(id, season, episode, percent, position) {
		this.sender.send({
			'Series': { id, season, episode, percent, position }
		});
	}

	close() {
		this.sender.close();
	}
}

export function newProgressStream() {
	const sender = stream.newSender('progress');
	return new ProgressStream(sender);
}