import { Writable } from 'chuchi/stores';

export type ContextMenuEvent = { clientX: number; clientY: number };
export type ContextMenuOpts = { id: string; text: string };
export type OnCloseFn = (id: string | null) => void;

type Inner = [ContextMenuEvent, ContextMenuOpts, OnCloseFn];

export default class ContextMenu {
	currentOpts: Writable<Inner | null>;

	constructor() {
		this.currentOpts = new Writable(null);
	}

	/// ev: { clientX, clientY }, opts: { id: "", text: "" }, onClose: id | null => {}
	open(ev: ContextMenuEvent, opts: ContextMenuOpts, onClose: OnCloseFn) {
		const curr = this.currentOpts.get();
		if (curr !== null) throw new Error('can only open contextmenu once');

		this.currentOpts.set([
			ev,
			opts,
			id => {
				this.currentOpts.set(null);
				onClose(id);
			},
		]);
	}
}
