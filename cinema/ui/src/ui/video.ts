import Listeners from 'chuchi-utils/sync/Listeners';

export function waitOnce<R>(listeners: Listeners<[R]>): Promise<R> {
	return new Promise(res => {
		const rmFn = listeners.add(t => {
			rmFn();
			res(t);
		});
	});
}

/// ```
/// const v = new Video;
/// v.bind(document.getElementById('video'));
/// const readyProm = v.waitReady();
/// v.setSrc(src);
/// v.setPosition(pos);
/// await readyProm;
/// ```
export default class Video {
	el: HTMLVideoElement;
	waitingOnMetadata: boolean;
	setPercentOnMetadataLoaded: number;

	canplayListeners: Listeners<[Event]>;
	errorListeners: Listeners<[Event]>;
	onPositionListeners: Listeners<[Event]>;

	constructor() {
		this.el = document.createElement('video');
		this.waitingOnMetadata = true;
		this.setPercentOnMetadataLoaded = 0;

		this.canplayListeners = new Listeners();
		this.errorListeners = new Listeners();
		this.onPositionListeners = new Listeners();
	}

	bind(cont: HTMLElement) {
		cont.appendChild(this.el);

		this.el.addEventListener('loadedmetadata', e => {
			this.waitingOnMetadata = false;
			this.el.currentTime =
				this.setPercentOnMetadataLoaded * this.el.duration;
		});
		this.el.addEventListener('canplay', e => {
			this.canplayListeners.trigger(e);
		});
		this.el.addEventListener('error', e => {
			console.log('error', e);
			this.errorListeners.trigger(e);
		});
		this.el.addEventListener('timeupdate', e => {
			if (!this.waitingOnMetadata) this.onPositionListeners.trigger(e);
		});
	}

	onPositionUpdate(fn: (e: Event) => void) {
		return this.onPositionListeners.add(fn);
	}

	/// returns a promise
	waitReady() {
		return new Promise((res, error) => {
			let rmError = () => {};
			let rmCanplay = () => {};
			rmCanplay = this.canplayListeners.add(() => {
				rmError();
				rmCanplay();
				res(undefined);
			});
			rmError = this.errorListeners.add(e => {
				rmCanplay();
				rmError();
				error(e);
			});
		});
	}

	assertReady() {
		if (this.waitingOnMetadata)
			throw new Error('canplay was not triggered');
	}

	setSrc(src: string, newPercent: number) {
		// the same source, let's just update the position
		if (this.el.src === src) {
			this.el.currentTime = newPercent * this.el.duration;
			return;
		}

		this.el.src = src;
		this.el.type = 'video/mp4';

		// the source was changed we need a canplay event before we should
		// trigger the timeupdate again
		this.waitingOnMetadata = true;
		// this might not be necessary but i'm on the safe side
		// the problem is this might triggered a timeupdate
		this.setPercentOnMetadataLoaded = newPercent;
	}

	position() {
		this.assertReady();
		return this.el.currentTime;
	}

	setPosition(pos: number) {
		this.assertReady();
		this.el.currentTime = pos;
	}

	len(): number {
		this.assertReady();
		return this.el.duration;
	}

	progress(): number {
		return this.position() / this.len();
	}

	setProgress(prog: number) {
		this.setPosition(prog * this.len());
	}

	remainingTime(): number {
		return this.len() - this.position();
	}

	isPlaying(): boolean {
		this.assertReady();
		return !this.el.paused;
	}

	play() {
		this.assertReady();
		this.el.play();
	}

	pause() {
		this.assertReady();
		this.el.pause();
	}

	forward(secs: number) {
		this.setPosition(Math.min(this.len(), this.position() + secs));
	}

	reverse(secs: number) {
		this.setPosition(Math.max(0, this.position() - secs));
	}
}

/*
<video>
	<source src="/assets/video/example.mp4" type="video/mp4">
</video>
*/
