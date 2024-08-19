import { Sender } from 'chuchi-core/api/Stream';
import DateTime from 'chuchi-legacy/time/DateTime';
import { padZero } from 'chuchi-utils';
import { Api, Stream } from 'chuchi/api';

export const MIN_PERCENT = 0.0001;
export const MAX_PERCENT = 0.9999;

// @ts-ignore
const addr = window.API_SERVER_ADDR;
const api = new Api(addr + 'api/cinema/');
export let stream = new Stream(api, '/stream');

stream.onClose(() => {
	console.log('stream closed');
	alert("E Fähler isch passiert. Bitte lad d'site neu");
});

export function assets(url: string): string {
	return `${addr}assets/cinema/${url}`;
}

export function bgImg(url: string): string {
	return `background-image: url("${assets(url)}")`;
}

export type EntryData =
	| { kind: 'Movie'; data: Movie }
	| { kind: 'Series'; data: Series };

export class Entry {
	id!: string;
	name!: string;
	originalName: string | null;
	description: string | null;
	rating: number | null;
	data: EntryData;
	updatedOn: DateTime;
	genres!: string[];

	constructor(d: any) {
		Object.assign(this, d);

		switch (this.data.kind) {
			case 'Movie':
				this.data = { kind: 'Movie', data: new Movie(this.data.data) };
				break;
			case 'Series':
				this.data = {
					kind: 'Series',
					data: new Series(this.data.data),
				};
				break;
		}

		this.updatedOn = new DateTime(d.updatedOn);
	}
}

export class Movie {
	duration: number | null;
	year: number;
	progress: Progress | null;

	constructor(d: any) {
		Object.assign(this, d);

		this.progress = d.progress ? new Progress(d.progress) : null;
	}

	// title(): string {
	// 	return this.name + ' ' + this.year;
	// }

	// poster(): string {
	// 	return assets(`posters/movies/${encodeURIComponent(this.title())}.jpg`);
	// }

	// fullPoster(): string {
	// 	return assets(
	// 		`full-posters/movies/${encodeURIComponent(this.title())}.jpg`,
	// 	);
	// }

	// src(): string {
	// 	return assets(`movies/${encodeURIComponent(this.title())}.mp4`);
	// }

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(totalLen: number): number {
		if (totalLen < 5 * 60) return 5;
		if (totalLen < 60 * 60) return 2 * 60;
		return 4 * 60;
	}

	// getUpdatedOn(): DateTime {
	// 	return this.updatedOn;
	// }

	progressUpdatedOn(): DateTime | null {
		return this.progress?.updatedOn ?? null;
	}

	percent(): number {
		return this.progress?.percent ?? 0;
	}

	position(): number {
		return this.progress?.position ?? 0;
	}

	setProgress(percent: number, position: number) {
		this.progress = new Progress({
			percent,
			position,
			updatedOn: new DateTime(),
		});
	}

	// sendProgress(stream: ProgressStream) {
	// 	const percent = this.progress?.percent ?? 0;
	// 	const position = this.progress?.position ?? 0;
	// 	stream.sendMovie(this.id, percent, position);
	// }
}

export class Series {
	seasons: Season[];

	constructor(d: any) {
		Object.assign(this, d);

		this.seasons = d.seasons.map((s: any) => new Season(s));
	}

	// poster(): string {
	// 	return assets(`posters/series/${encodeURIComponent(this.name)}.jpg`);
	// }

	// fullPoster(): string {
	// 	return assets(
	// 		`full-posters/series/${encodeURIComponent(this.name)}.jpg`,
	// 	);
	// }

	// src(seasonIdx: number, episodeIdx: number): string {
	// 	const seas = this.seasons[seasonIdx];
	// 	const season = encodeURIComponent(seas.folderName(seasonIdx));
	// 	const ep = seas.episodes[episodeIdx];
	// 	const episode = encodeURIComponent(ep.fileName(episodeIdx));
	// 	return assets(`series/${this.name}/${season}/${episode}`);
	// }

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(
		seasonIdx: number,
		episodeIdx: number,
		totalLen: number,
	): number {
		return this.seasons[seasonIdx].episodes[episodeIdx].creditsDuration(
			totalLen,
		);
	}

	getUpdatedOn(): DateTime {
		let latest = null;
		this.seasons.forEach(season => {
			season.episodes.forEach(ep => {
				const updatedOn = ep.getUpdatedOn();
				if ((updatedOn && !latest) || latest.time < updatedOn.time)
					latest = updatedOn;
			});
		});

		if (!latest) throw new Error('should never be null');

		return latest;
	}

	progressUpdatedOn(): DateTime | null {
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

	percent(seasonIdx: number, episodeIdx: number): number {
		return this.seasons[seasonIdx].episodes[episodeIdx].percent();
	}

	position(seasonIdx: number, episodeIdx: number): number {
		return this.seasons[seasonIdx].episodes[episodeIdx].position();
	}

	setProgress(
		seasonIdx: number,
		episodeIdx: number,
		percent: number,
		position: number,
	) {
		this.seasons[seasonIdx].episodes[episodeIdx].setProgress(
			percent,
			position,
		);
	}

	// sendProgress(
	// 	stream: ProgressStream,
	// 	seasonIdx: number,
	// 	episodeIdx: number,
	// ) {
	// 	const ep = this.seasons[seasonIdx].episodes[episodeIdx];

	// 	const percent = ep.progress?.percent ?? 0;
	// 	const position = ep.progress?.position ?? 0;
	// 	stream.sendSeries(this.id, seasonIdx, episodeIdx, percent, position);
	// }
}

export class Progress {
	percent: number;
	updatedOn: DateTime;

	constructor(d: any) {
		Object.assign(this, d);

		this.updatedOn = new DateTime(d.updatedOn);
	}

	isCompleted(): boolean {
		return this.percent > MAX_PERCENT;
	}
}

export class Season {
	id!: string;
	season: number;
	name!: string | null;
	originalName!: string | null;
	episodes: Episode[];

	constructor(d: any) {
		Object.assign(this, d);

		this.episodes = d.episodes.map((e: any) => new Episode(e));
	}

	title(): string {
		let name = this.name ? ' ' + this.name : '';
		return 'Season ' + padZero(this.season) + name;
	}

	folderName(): string {
		return this.title();
	}

	totalPercent(): number {
		const l = this.episodes.length;
		return this.episodes.reduce((t, ep) => ep.percent() / l + t, 0);
	}
}

export class Episode {
	id!: string;
	episode!: number;
	name!: string;
	originalName!: string | null;
	updatedOn: DateTime;
	progress: Progress | null;

	constructor(d: any) {
		Object.assign(this, d);

		this.updatedOn = new DateTime(d.updatedOn);
		this.progress = d.progress ? new Progress(d.progress) : null;
	}

	isCompleted(): boolean {
		return this.progress && this.progress.isCompleted();
	}

	title(idx: number): string {
		return `${padZero(idx + 1)} - ${this.name}`;
	}

	fileName(idx: number): string {
		return `Episode ${padZero(idx + 1)} ${this.name}.mp4`;
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(totalLen: number): number {
		if (totalLen < 5 * 60) return 5;
		return 20;
	}

	getUpdatedOn(): DateTime {
		return this.updatedOn;
	}

	progressUpdatedOn(): DateTime | null {
		return this.progress?.updatedOn ?? null;
	}

	percent(): number {
		return this.progress?.percent ?? 0;
	}

	setProgress(percent: number, position: number) {
		this.progress = new Progress({
			percent,
			position,
			updatedOn: new DateTime(),
		});
	}
}

export class EntriesResp {
	list: Entry[];

	constructor(d: any) {
		this.list = d.list.map((e: any) => new Entry(e));
	}
}

// returns all entries
export async function entries(token: string): Promise<EntriesResp> {
	const d = await api.request(
		'GET',
		'entries',
		null,
		{ 'auth-token': token },
		{ credentials: 'include' },
	);
	return new EntriesResp(d);
}

export class ProgressStream {
	sender: Sender;

	constructor(sender: Sender) {
		this.sender = sender;
	}

	async open(token: string) {
		if (!stream.isConnect()) stream.connect();

		await this.sender.open({ token });
	}

	sendMovie(id: string, percent: number, position: number) {
		this.sender.send({
			Movie: { id, percent, position },
		});
	}

	sendSeries(
		id: string,
		season: number,
		episode: number,
		percent: number,
		position: number,
	) {
		this.sender.send({
			Series: { id, season, episode, percent, position },
		});
	}

	close() {
		this.sender.close();
	}
}

export function newProgressStream(): ProgressStream {
	const sender = stream.newSender('progress');
	return new ProgressStream(sender);
}
