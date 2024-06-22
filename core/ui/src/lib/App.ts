import { Core } from 'core-lib';

// @ts-ignore
const addr = import.meta.env.SERVER_ADDR;

export default class App {
	key: string;

	constructor(key: string) {
		this.key = key;
	}

	uri(): string {
		return '/' + this.key;
	}

	icon(): string {
		return `${addr}assets/${this.key}/icon.png`;
	}

	name() {
		throw new Error('todo name');
	}

	async prepare(cl: Core) {}

	init(cl: Core) {}
}
