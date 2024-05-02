module.exports = {
	extraFiles: ["../rust_controller/pkg/levandevideo_rust_bg.wasm"],
}


/* Config for the companion conf, in case it gets nuked:
module: {
		rules: [
			{
				test: /\.node$/,
				type: 'node-loader',
			},
			{
				test: /\.wasm$/,
				type: 'asset/resource',
			},
			{
				test: /\.ts$/,
				type: "ts-loader"
			},


			// {
			// 	test: /\.json$/,
			// 	type: 'asset/source',
			// },
			// {
			// 	test: /BUILD$/,
			// 	type: 'asset/resource',
			// 	generator: {
			// 		filename: 'BUILD',
			// 	},
			// },
		],
	},
 */