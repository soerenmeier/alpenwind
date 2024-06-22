export class ComponentBuilder {
	/// returns a function which detaches again
	attach(
		el: HTMLElement,
		props: any = {},
		context: Map<string, string> | null = null,
	): () => void {
		throw new Error('todo');
	}
}

export class SvelteComponent extends ComponentBuilder {
	private _comp: any;
	private _props: any;

	/// note these props can be overriden by search queries
	constructor(comp: any, props: any = {}) {
		super();

		this._comp = comp;
		this._props = props;
	}

	attach(
		el: HTMLElement,
		props: any = {},
		context: Map<string, string> | null = null,
	): () => void {
		if (!context) context = new Map();

		const comp = new this._comp({
			target: el,
			props: { ...this._props, ...props },
			context,
			intro: true,
		});
		return () => comp.$destroy();
	}
}
