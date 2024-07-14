import { SessionStore, User, newState } from './user';
import ContextMenu from './ContextMenu';
import { Router } from 'chuchi';
import { Writable } from 'chuchi/stores';

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
