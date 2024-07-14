import * as routing from './routing';
import * as user from './user';
import Core from './Core';
import { Route } from 'chuchi';

const router = {
	Route,
	ComponentBuilder: routing.ComponentBuilder,
	SvelteComponent: routing.SvelteComponent,
};

export { router, user };

export type { Core };

export type CoreArgs = {
	context: Map<any, any>;
	getContext: (key: string) => any;
};

export function newCore(args: CoreArgs): Core {
	const cl = new Core();
	args.context.set('core-lib', cl);
	globalThis.coreLibGetContext = args.getContext;

	return cl;
}

export function getCore(): Core {
	return globalThis.coreLibGetContext('core-lib');
}
