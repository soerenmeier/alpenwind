// example: http://localhost:6120/
export let serverAddr = null;

/// should only be used by core
export function init(addr) {
	serverAddr = addr;
}