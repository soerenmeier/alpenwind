import Router from 'fire-svelte/routing/Router';
import { SessionStore, User, newState } from './user';
import Writable from 'fire-svelte/stores/Writable';
import ContextMenu from './ContextMenu';

export default class Core {
	router: Router;
	session: SessionStore;
	user: Writable<User>;
	contextMenu: ContextMenu;

	constructor() {
		this.router = new Router();

		const userState = newState();
		this.session = userState.session;
		this.user = userState.user;

		this.contextMenu = new ContextMenu();
	}
}
