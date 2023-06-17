import Listeners from 'fire/util/listeners.js';

const manualListeners = new Listeners;

export default class Router {
	constructor() {
		this.routes = [];

		this._currentReq = Request.fromCurrent();
		this._listen();
		this._changedHistory = false;
		this.routeChangeListeners = new Listeners;

		window.history.replaceState(this._currentReq.toHistoryState(), '');
	}

	init() {
		this.routeChangeListeners.trigger(this.matchRoute(this._currentReq));
	}

	addRoute(route) {
		this.routes.push(route);
	}

	onRouteChange(fn) {
		return this.routeChangeListeners.add(fn);
	}

	matchRoute(req) {
		return this.routes.find(r => r.check(req)) ?? null;
	}

	currentReq() {
		return this._currentReq;
	}

	currentState() {
		return this.currentReq().state ?? {};
	}

	replaceState(state) {
		this._currentReq.state = state;
		window.history.replaceState(this._currentReq.toHistoryState(), '');
	}

	// this only works until the page get's reloaded
	canGoBack() {
		return this._changedHistory;
	}

	back() {
		window.history.back();
	}

	open(url, state = {}) {
		manualListeners.trigger({ url, state });
	}

	// ignores the request if we already have opened this page
	_openReq(req) {
		if (this._currentReq.uri === req.uri)
			return;

		this._currentReq = req;
		window.history.pushState(req.toHistoryState(), '', req.uri);
		this._changedHistory = true;

		this.routeChangeListeners.trigger(this.matchRoute(req));
	}

	_listen() {
		window.addEventListener('click', e => {
			const link = e.composedPath().find(e => {
				return e.tagName && e.tagName.toLowerCase() === 'a';
			});

			if (!link)
				return;

			const uri = this._convertUrlToUri(link.href);
			if (uri === null)
				return;

			e.preventDefault();

			this._openReq(new Request(uri));
		});

		manualListeners.add(({ url, state }) => {
			let uri = url;
			if (!url.startsWith('/')) {
				uri = this._convertUrlToUri(url);
				if (uri === null)
					throw new Error('url not parseable: ' + url);
			}

			this._openReq(new Request(uri, state));
		})

		window.addEventListener('popstate', e => {
			e.preventDefault();

			const req = new Request(e.state.uri, e.state.state ?? {});
			this._currentReq = req;

			this.routeChangeListeners.trigger(this.matchRoute(req));
		});
	}

	/// returns null if the url does not match our host and protocol
	_convertUrlToUri(url) {
		const loc = window.location;
		const protHost = loc.protocol + '//' + loc.host + '/';
		if (!url.startsWith(protHost))
			return null;

		return '/' + url.slice(protHost.length);
	}
}

function sanitizeUri(uri) {
	if (uri.length < 2)
		return uri;

	if (uri[uri.length - 1] === '/')
		return uri.substr(0, uri.length - 1);
	return uri;
}

export class Request {
	constructor(uri, state = {}) {
		this.uri = sanitizeUri(uri);
		this.state = state;
	}

	static fromCurrent() {
		const state = window.history.state?.state ?? null;
		return new Request(window.location.pathname, state);
	}

	toHistoryState() {
		return {
			uri: this.uri,
			state: this.state
		};
	}
}

export class Route {
	/// returns true if this route matches the request
	check(req) {
		return false;
	}

	/// el: a dom element where this routes component should be inserted
	/// needs to return a function which when called removes the route
	/// from the dom element
	attachComponent(el) {
		throw new Error('not implemented');
	}
}

export class StaticRoute extends Route {
	/// needs to be a ComponentBuilder
	constructor(uri, compBuilder) {
		super();

		this.uri = uri;
		this._comp = compBuilder;
	}

	check(req) {
		return req.uri === this.uri;
	}

	attachComponent(el) {
		return this._comp.attach(el);
	}
}

export class ComponentBuilder {
	/// returns a function which detaches again
	attach(el) {
		throw new Error('todo');
	}
}

export class SvelteComponent extends ComponentBuilder {
	constructor(comp, props = {}) {
		super();

		this._comp = comp;
		this._props = props;
	}

	attach(el) {
		const comp = new this._comp({
			target: el,
			props: this._props,
			intro: true
		});
		return () => comp.$destroy();
	}
}

export const router = new Router;