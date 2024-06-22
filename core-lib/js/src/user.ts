import Writable from 'fire-svelte/stores/Writable';
import ApiError from 'fire/api/ApiError';
import Listeners from 'fire/sync/Listeners';
import DateTime from 'fire/time/DateTime';

export class User {
	id!: string;
	username!: string;
	name!: string;
	rights!: any;

	constructor(d: any) {
		Object.assign(this, d);
	}
}

export class Session {
	token!: string;
	dataToken!: string;
	timeout!: number;
	createdOn!: DateTime;
	userId!: string;

	constructor(d: any) {
		Object.assign(this, d);
		this.createdOn = new DateTime(d.createdOn);
	}

	isValid() {
		return this.timeout * 1000 > Date.now();
	}
}

export class SessionStore {
	sess: Session | null;
	listeners: Listeners<[Session | null]>;

	constructor() {
		this.sess = null;

		// load from localStorage
		try {
			const itm = localStorage.getItem('auth-session');
			if (itm) {
				const s = JSON.parse(itm);
				const sess = new Session(s);
				if (sess.isValid()) this.sess = sess;
			}
		} catch (e) {}

		this.listeners = new Listeners();
	}

	subscribe(fn: (v: Session | null) => void) {
		fn(this.sess);
		return this.listeners.add(fn);
	}

	get(): Session | null {
		return this.sess;
	}

	existsAndIsValid(): boolean {
		return this.sess && this.sess.isValid();
	}

	// throws an API error if the session does not exists
	// or if it is not valid anymore
	getValid(): Session {
		if (!this.sess)
			throw new ApiError('MissingAuthToken', 'no session present');
		if (!this.sess.isValid())
			throw new ApiError('InvalidAuthToken', 'session expired');

		return this.sess;
	}

	// v: Session
	set(v: Session) {
		this.sess = v;

		if (v) localStorage.setItem('auth-session', JSON.stringify(v));
		else localStorage.removeItem('auth-session');

		this.listeners.trigger(v);
	}
}

// returns { session, user }
export function newState(): {
	session: SessionStore;
	user: Writable<User | null>;
} {
	return {
		// of type api/users/Session
		session: new SessionStore(),
		// of type api/users/User
		user: new Writable(null),
	};
}
