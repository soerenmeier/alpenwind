import type { InputOptions, OutputOptions, RollupOptions } from 'rollup';

import typescriptPlugin from '@rollup/plugin-typescript';
import terserPlugin from '@rollup/plugin-terser';
import dtsPlugin from 'rollup-plugin-dts';
import { nodeResolve } from '@rollup/plugin-node-resolve';

const outputPath = 'dist/corelib';
const commonInputOptions: InputOptions = {
	input: 'src/index.ts',
	plugins: [typescriptPlugin(), nodeResolve()],
};
const iifeCommonOutputOptions: OutputOptions = {
	name: 'corelib',
};

const config: RollupOptions[] = [
	{
		...commonInputOptions,
		output: [
			{
				file: `${outputPath}.js`,
				format: 'esm',
			},
			{
				file: `${outputPath}.min.js`,
				format: 'esm',
				plugins: [terserPlugin()],
			},
		],
	},
	{
		...commonInputOptions,
		plugins: [commonInputOptions.plugins, dtsPlugin()],
		output: [
			{
				file: `${outputPath}.d.ts`,
				format: 'esm',
			},
		],
	},
];

export default config;
