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
      callback: (
        movedModules,
        chunks,
        next,
        addNewChunk,
        removeModuleFromChunk
      ) => {
        console.log("📦 Modules moved to vendors chunk:", movedModules);
        console.log(`Total modules moved: ${movedModules.length}`);

        // You can do anything with the moved modules here
        movedModules.forEach((module, index) => {
          console.log(`  ${index + 1}. ${module}`);
        });

        // Display all chunks and their modules
        console.log("\n🔍 All chunks and their modules:");
        chunks.forEach(([chunkName, modules], index) => {
          console.log(
            `  ${index + 1}. Chunk: "${chunkName}" (${modules.length} modules)`
          );
          modules.forEach((module, moduleIndex) => {
            console.log(`     ${moduleIndex + 1}. ${module}`);
          });
        });

        // Example: Analyze chunks and make decisions based on their content
        console.log("\n📊 Chunk Analysis:");
        chunks.forEach(([chunkName, modules]) => {
          const nodeModulesCount = modules.filter((module) =>
            module.includes("node_modules")
          ).length;
          const appModulesCount = modules.filter(
            (module) => !module.includes("node_modules")
          ).length;

          console.log(`  Chunk "${chunkName}":`);
          console.log(`    - Total modules: ${modules.length}`);
          console.log(`    - Node modules: ${nodeModulesCount}`);
          console.log(`    - App modules: ${appModulesCount}`);

          // Example: If a chunk has too many node_modules, split it
          if (nodeModulesCount > 0 && appModulesCount > 0) {
            console.log(
              `    ⚠️  Mixed chunk detected - contains both app and node_modules`
            );
          }
        });

        // Example: Create a new chunk with some modules
        console.log("\n🔧 Creating a new chunk with some modules...");
        const modulesForNewChunk = movedModules.slice(0, 2); // Take first 2 modules
        addNewChunk("custom-chunk", modulesForNewChunk);
        console.log(
          `✅ Created new chunk 'custom-chunk' with ${modulesForNewChunk.length} modules`
        );

        // Example: Remove some modules from vendors chunk
        console.log("\n🗑️ Removing some modules from vendors chunk...");
        const modulesToRemove = movedModules.slice(0, 1); // Remove first module
        removeModuleFromChunk("vendors", modulesToRemove);
        console.log(
          `✅ Removed ${modulesToRemove.length} modules from 'vendors' chunk`
        );

        // Example: Find specific modules across chunks
        console.log("\n🔎 Finding specific modules across chunks:");
        const targetModule = movedModules[0]; // First moved module
        if (targetModule) {
          const moduleName = path.basename(targetModule);
          console.log(`  Looking for modules containing "${moduleName}":`);

          chunks.forEach(([chunkName, modules]) => {
            const matchingModules = modules.filter((module) =>
              path.basename(module).includes(moduleName)
            );
            if (matchingModules.length > 0) {
              console.log(
                `    Found in chunk "${chunkName}": ${matchingModules.length} modules`
              );
            }
          });
        }

        // Simulate some async work
        console.log("\n⏳ Simulating some async work...");
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
