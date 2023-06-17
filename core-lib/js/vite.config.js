import { resolve } from 'path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import esbuild from 'esbuild';

// https://vitejs.dev/config/
export default defineConfig({
	build: {
		lib: {
			entry: resolve(__dirname, 'src/lib.js'),
			name: 'CoreLib',
			formats: ['es'],
			fileName: 'corelib'
		}
	},
	plugins: [
		svelte(),
		{
			name: 'minify',
			closeBundle: async () => {
				esbuild.buildSync({
					entryPoints: ['./dist/corelib.js'],
					minify: true,
					allowOverwrite: true,
					outfile: './dist/corelib.js'
				})
			}
		}
	]
});