const path = require("node:path");

const rspack = require("@rspack-template/core");

const compiler = rspack({
  context: __dirname,
  mode: "development",
  cache: false, // Disable cache to force re-generation
  entry: {
    main: "./src/index.js",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
  },
  plugins: [
    new rspack.MyBannerPlugin({
      chunkName: "vendors",
      callback: (movedModules, next) => {
        console.log("📦 Modules moved to vendors chunk:", movedModules);
        console.log(`Total modules moved: ${movedModules.length}`);

        // You can do anything with the moved modules here
        movedModules.forEach((module, index) => {
          console.log(`  ${index + 1}. ${module}`);
        });

        // Simulate some async work
        console.log("⏳ Simulating some async work...");
        setTimeout(() => {
          console.log(
            "✅ Async work completed, calling next() to resume plugin execution"
          );
          next(); // Resume the plugin execution
        }, 2000);
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
