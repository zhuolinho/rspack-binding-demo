const path = require("node:path");

const rspack = require("@rspack-template/core");

const compiler = rspack({
  context: __dirname,
  mode: "development",
  entry: {
    main: "./src/index.js",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
  },
  plugins: [
    new rspack.MyBannerPlugin({
      chunkName: "vendors",
      callback: (movedModules) => {
        console.log("📦 Modules moved to vendors chunk:", movedModules);
        console.log(`Total modules moved: ${movedModules.length}`);

        // You can do anything with the moved modules here
        movedModules.forEach((module, index) => {
          console.log(`  ${index + 1}. ${module}`);
        });
      },
    }),
  ],
});

compiler.run((err, stats) => {
  if (err) {
    console.error(err);
  }
  console.info(stats.toString({ colors: true }));
});
