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
		this.routeChangeListeners.trigger(
			this.matchRoute(this._currentReq),
			this._currentReq
		);
	}

	addRoute(route) {
		this.routes.push(route);
	}

	onRouteChange(fn) {
		return this.routeChangeListeners.add(fn);
	}

	/// returns an AttachableRoute or null
	matchRoute(req) {
		for (const route of this.routes) {
			const attachable = route.check(req);
			if (attachable)
				return attachable;
		}

		return null;
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

	replaceReq(req) {
		this._currentReq = req;
		window.history.replaceState(
			req.toHistoryState(),
			'',
			req.toUriWithSearch()
		);
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

	/// This is only intended to be used if you wan't to modify the history state
	/// without triggering a routeChange Event
	pushReq(req) {
		this._currentReq = req;
		window.history.pushState(
			req.toHistoryState(),
			'',
			req.toUriWithSearch()
		);
		this._changedHistory = true;
	}

	// ignores the request if we already have opened this page
	_openReq(req) {
		if (this._currentReq.toUriWithSearch() === req.toUriWithSearch())
			return;

		this._currentReq = req;
		window.history.pushState(
			req.toHistoryState(),
			'',
			req.toUriWithSearch()
		);
		this._changedHistory = true;

		this.routeChangeListeners.trigger(this.matchRoute(req), req);
	}

	_listen() {
		window.addEventListener('click', e => {
			const link = e.target.closest('a');
			const openInNewTab = e.metaKey || e.ctrlKey || e.shiftKey;
			const saveLink = e.altKey;
			if (!link || !link.href || openInNewTab || saveLink)
				return;

			const target = link?.target ?? '';
			if (target.toLowerCase() === '_blank')
				return;

			const req = this._urlToRequest(link.href);
			if (!req)
				return;

			e.preventDefault();

			this._openReq(req);
		});

		manualListeners.add(({ url, state }) => {
			const req = this._urlToRequest(url, state);
			if (!req)
				throw new Error('url not parseable: ' + url);

			this._openReq(req);
		})

		window.addEventListener('popstate', e => {
			e.preventDefault();

			const req = new Request(
				e.state.uri,
				e.state.state ?? {},
				e.state?.search ?? ''
			);
			this._currentReq = req;

			this.routeChangeListeners.trigger(this.matchRoute(req));
		});
	}

	/// returns null if the url does not match our host and protocol
	_urlToRequest(url, state = {}) {
		const loc = window.location;

		if (url.startsWith('/'))
			url = loc.protocol + '//' + loc.host + url;

		url = new URL(url);
		// validate protocol and host
		if (url.protocol !== loc.protocol || url.host !== loc.host)
			return null;

		return new Request(url.pathname, state, url.search);
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
	constructor(uri, state = {}, search = '') {
		this.uri = sanitizeUri(uri);
		this.search = new URLSearchParams(search);
		this.state = state;
	}

	static fromCurrent() {
		return new Request(
			window.location.pathname,
			window.history.state?.state ?? null,
			window.location.search
		);
	}

	toUriWithSearch() {
		let uri = this.uri;
		if (this.search.size)
			uri += '?' + this.search.toString();

		return uri;
	}

	toHistoryState() {
		return {
			uri: this.uri,
			state: this.state,
			search: this.search.toString()
		};
	}

	/// does only copy the state on level deep
	clone() {
		const { uri, state, search } = this.toHistoryState();
		return new Request(uri, { ...state }, search);
	}
}

export class Route {
	/// returns an AttachableRoute if this route matches the request
	/// else null
	check(req) {
		return null;
	}
}

export class StaticRoute extends Route {
	/// needs to be a ComponentBuilder
	/// uri can either be a regex or a string
	constructor(uri, compBuilder) {
		super();

		this.uri = uri;
		this._comp = compBuilder;
	}

	check(req) {
		let props = {};

		if (typeof this.uri === 'string') {
			if (this.uri !== req.uri)
				return;
		} else {
			const match = req.uri.match(this.uri);
			if (!match || match[0] != req.uri)
				return false;

			props = match.groups;
		}

		const searchObj = Object.fromEntries(req.search);
		props = { ...searchObj, ...props };

		return new AttachableRoute(props, this._comp);
	}
}

export class AttachableRoute {
	constructor(props, compBuilder) {
		this._props = props;
		this._comp = compBuilder;
	}

	/// el: a dom element where this routes component should be inserted
	/// needs to return a function which when called removes the route
	/// from the dom element
	attachComponent(el, context = null) {
		return this._comp.attach(el, this._props, context);
	}
}

export class ComponentBuilder {
	/// returns a function which detaches again
	attach(el, props = {}, context = null) {
		throw new Error('todo');
	}
}

export class SvelteComponent extends ComponentBuilder {
	/// note these props can be overriden by search queries
	constructor(comp, props = {}) {
		super();

		this._comp = comp;
		this._props = props;
	}

	attach(el, props = {}, context = null) {
		if (!context)
			context = new Map;

		const comp = new this._comp({
			target: el,
			props: { ...this._props, ...props },
			context,
			intro: true
		});
		return () => comp.$destroy();
	}
}