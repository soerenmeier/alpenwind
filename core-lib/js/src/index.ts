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

export type CoreArgs = {};

export function newCore(args: CoreArgs = {}): Core {
	const cl = new Core();

	globalThis.CORE_LIB_01 = cl;

	return cl;
}

export function getCore(): Core {
	// since we are cross npm project we might have different svelte versions, and so can't use getContext
	// but since it is a singleton anyway i think this should be fine'
	return globalThis.CORE_LIB_01;
}
