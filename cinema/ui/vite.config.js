import { resolve } from 'path';
import { renameSync } from 'fs';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import replace from '@rollup/plugin-replace';
import esbuild from 'esbuild';
import { randomToken } from 'fire/util.js';

// https://vitejs.dev/config/
export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/main.js'),
			name: 'Cinema',
			formats: ['es'],
			fileName: 'main'
		},
		rollupOptions: {
			external: ['core-lib']
		}
	},
	plugins: [
		svelte(),
		{
			name: 'minify',
			closeBundle: () => {
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