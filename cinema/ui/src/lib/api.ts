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

		switch (d.data.kind) {
			case 'Movie':
				this.data = { kind: 'Movie', data: new Movie(d.data) };
				break;
			case 'Series':
				this.data = {
					kind: 'Series',
					data: new Series(d.data),
				};
				break;
		}

		this.updatedOn = new DateTime(d.updatedOn);
	}

	get kind(): 'Movie' | 'Series' {
		return this.data.kind;
	}

	posterUrl(): string {
		return this.data.data.posterUrl(this.name);
	}

	fullPosterUrl(): string {
		return this.data.data.fullPosterUrl(this.name);
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

	private encodedTitle(name: string): string {
		return encodeURIComponent(name + ' ' + this.year);
	}

	posterUrl(name: string): string {
		return assets(`posters/movies/${this.encodedTitle(name)}.jpg`);
	}

	fullPosterUrl(name: string): string {
		return assets(`full-posters/movies/${this.encodedTitle(name)}.jpg`);
	}

	videoSrc(name: string): string {
		return assets(`movies/${this.encodedTitle(name)}.mp4`);
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(duration: number): number {
		if (duration < 5 * 60) return 5;
		if (duration < 60 * 60) return 2 * 60;
		return 4 * 60;
	}

	setProgress(percent: number) {
		this.progress = Progress.new(percent);
	}
}

export class Series {
	seasons: Season[];

	constructor(d: any) {
		Object.assign(this, d);

		this.seasons = d.seasons.map((s: any) => new Season(s));
	}

	posterUrl(name: string): string {
		return assets(`posters/series/${encodeURIComponent(name)}.jpg`);
	}

	fullPosterUrl(name: string): string {
		return assets(`full-posters/series/${encodeURIComponent(name)}.jpg`);
	}

	videoSrc(name: string, seasonIdx: number, episodeIdx: number): string {
		const seas = this.seasons[seasonIdx];
		const season = encodeURIComponent(seas.folderName());
		const ep = seas.episodes[episodeIdx];
		const episode = encodeURIComponent(ep.fileName());
		return assets(`series/${name}/${season}/${episode}`);
	}
}

export class Progress {
	percent: number;
	updatedOn: DateTime;

	constructor(d: any) {
		Object.assign(this, d);

		this.updatedOn = new DateTime(d.updatedOn);
	}

	static new(percent: number): Progress {
		const prog = Object.create(Progress.prototype);
		prog.percent = percent;
		prog.updatedOn = new DateTime();

		return prog;
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

	// title(): string {}

	folderName(): string {
		let name = this.name ? ' ' + this.name : '';
		return 'Season ' + padZero(this.season) + name;
	}

	totalPercent(): number {
		const l = this.episodes.length;
		return this.episodes.reduce((t, ep) => ep.percent() / l + t, 0);
	}

	episodeById(id: string): Episode | undefined {
		return this.episodes.find(ep => ep.id === id);
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

	fileName(): string {
		return `Episode ${padZero(this.episode)} ${this.name}.mp4`;
	}

	/// the total Len is required to calculate the automatic creditsDuration
	/// totalLen in secs
	creditsDuration(totalLen: number): number {
		if (totalLen < 5 * 60) return 5;
		return 20;
	}

	percent(): number {
		return this.progress?.percent ?? 0;
	}

	setProgress(percent: number) {
		this.progress = Progress.new(percent);
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

export type ProgressId = {
	kind: 'Movie' | 'Episode';
	id: string;
};

export class ProgressStream {
	sender: Sender;

	constructor(sender: Sender) {
		this.sender = sender;
	}

	async open(token: string) {
		if (!stream.isConnect()) stream.connect();

		await this.sender.open({ token });
	}

	send(id: ProgressId, percent: number) {
		this.sender.send({ id, percent });
	}

	close() {
		this.sender.close();
	}
}

export function newProgressStream(): ProgressStream {
	const sender = stream.newSender('progress');
	return new ProgressStream(sender);
}
