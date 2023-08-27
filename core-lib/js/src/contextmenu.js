import { writable, get as getStore } from 'svelte/store';

export default class ContextMenu {
	constructor() {
		this.currentOpts = writable(null);
	}

	/// ev: { clientX, clientY }, opts: { id: "", text: "" }, onClose: id | null => {}
	open(ev, opts, onClose) {
		const curr = getStore(this.currentOpts);
		if (curr !== null)
			throw new Error('can only open contextmenu once');

		this.currentOpts.set([ev, opts, id => {
			this.currentOpts.set(null);
			onClose(id);
		}]);
	}
}