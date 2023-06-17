const addr = import.meta.env.SERVER_ADDR;

export default class App {
	constructor(key) {
		this.key = key;
	}

	uri() {
		return '/' + this.key;
	}

	icon() {
		return `${addr}assets/${this.key}/icon.png`;
	}

	name() {
		throw new Error('todo name');
	}

	async prepare() {}

	init() {}
}