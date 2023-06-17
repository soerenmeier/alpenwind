import { get as getStore, writable } from 'svelte/store';
import Data from 'fire/data/data.js';
import ApiError from 'fire/api/error.js';
import Listeners from 'fire/util/listeners.js';

export class User extends Data {
	constructor(d) {
		super({
			id: 'uid',
			username: 'str',
			name: 'str',
			rights: 'any'
		}, d);
	}
}

export class Session extends Data {
	constructor(d) {
		super({
			token: 'str',
			dataToken: 'str',
			timeout: 'int',
			createdOn: 'datetime',
			userId: 'uid'
		}, d);
	}

	isValid() {
		return this.timeout * 1000 > Date.now();
	}
}

class SessionStore {
	constructor() {
		this.sess = null;

		// load from localStorage
		try {
			const itm = localStorage.getItem('auth-session');
			if (itm) {
				const s = JSON.parse(itm);
				const sess = new Session(s);
				if (sess.isValid())
					this.sess = sess;
			}
		} catch (e) {}

		this.listeners = new Listeners;
	}

	subscribe(fn) {
		fn(this.sess);
		return this.listeners.add(fn);
	}

	get() {
		return this.sess;
	}

	existsAndIsValid() {
		return this.sess && this.sess.isValid();
	}

	// throws an API error if the session does not exists
	// or if it is not valid anymore
	getValid() {
		if (!this.sess)
			throw new ApiError('MissingAuthToken', 'no session present');
		if (!this.sess.isValid())
			throw new ApiError('InvalidAuthToken', 'session expired');

		return this.sess;
	}

	// v: Session
	set(v) {
		this.sess = v;

		if (v)
			localStorage.setItem('auth-session', JSON.stringify(v));
		else
			localStorage.removeItem('auth-session');

		this.listeners.trigger(v);
	}
}

// of type api/users/Session
export const session = new SessionStore;
// of type api/users/User
export const user = writable(null);
