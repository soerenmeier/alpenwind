import { resolve } from 'path';
import { readFileSync } from 'fs';
import { defineConfig } from 'vite';
import toml from 'toml';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import replace from '@rollup/plugin-replace';
import sveltePreprocess from 'svelte-preprocess';

const configStr = readFileSync('./../server/config.toml');
const config = toml.parse(configStr);

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