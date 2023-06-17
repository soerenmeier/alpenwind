import { writable, get as getStore } from 'svelte/store';

export const currentOpts = writable(null);

/// ev: { clientX, clientY }, opts: { id: "", text: "" }, onClose: id | null => {}
export function open(ev, opts, onClose) {
	const curr = getStore(currentOpts);
	if (curr !== null)
		throw new Error('can only open contextmenu once');

	currentOpts.set([ev, opts, id => {
		currentOpts.set(null);
		onClose(id);
	}]);
}