import { resolve } from 'path';
import { renameSync } from 'fs';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import esbuild from 'esbuild';
import { randomToken } from 'fire/util.js';
import sveltePreprocess from 'svelte-preprocess';

// https://vitejs.dev/config/
export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.js'),
			name: 'PwVault',
			formats: ['es'],
			fileName: 'main'
		},
		rollupOptions: {
			external: ['core-lib']
		}
	},
	plugins: [
		svelte({
			preprocess: sveltePreprocess()
		}),
		{
			name: 'minify',
			closeBundle: async () => {
				esbuild.buildSync({
					entryPoints: ['./dist/main.js'],
					minify: true,
					allowOverwrite: true,
					outfile: './dist/main.js'
				});

				const rnd = randomToken(5).toLowerCase();

				renameSync('./dist/main.js', './dist/main.' + rnd + '.js');
				renameSync('./dist/style.css', './dist/style.' + rnd + '.css');
			}
		}
	]
});