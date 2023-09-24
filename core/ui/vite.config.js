import { resolve } from 'path';
import { readFileSync } from 'fs';
import { defineConfig } from 'vite';
import toml from 'toml';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import replace from '@rollup/plugin-replace';
import sveltePreprocess from 'svelte-preprocess';

let config = {};
try {
	const configStr = readFileSync('./../server/config.toml');
	config = toml.parse(configStr);
} catch (e) {
	console.log('could not find config');
}

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
	let addr = '"http://' + config['listen-on'] + '/"';
	if (mode === 'production')
		addr = 'window.location.origin + "/"';

	return {
		build: {
			rollupOptions: {
				external: ['core-lib']
			}
		},
		plugins: [
			svelte({
				preprocess: sveltePreprocess()
			}),
			replace({
				preventAssignment: true,
				'import.meta.env.SERVER_ADDR': addr
			})
		]
	};
});