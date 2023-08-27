import Router, * as router from './router.js';
import * as user from './user.js';
import ContextMenu from './contextmenu.js';

export { router, user };

export default class CoreLib {
	constructor() {
		this.router = new Router;

		const userState = user.newState();
		this.session = userState.session;
		this.user = userState.user;

		this.contextMenu = new ContextMenu;
	}
}