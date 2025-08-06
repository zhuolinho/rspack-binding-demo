# Rspack binding template

**🚀 Unlock native Rust speed for Rspack — supercharge your builds, keep every JS feature, zero compromise, no limits.**

## Features

### MyBannerPlugin (Vendors Chunk Plugin)

A native plugin that creates a vendors chunk and automatically moves modules from `node_modules` into it, **without requiring `optimization.splitChunks` configuration**.

#### Usage

```javascript
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
```

#### How it works

1. Creates a new chunk named "vendors" (or whatever name you provide)
2. Scans all modules in the compilation during the `CompilationOptimizeChunkModules` phase
3. Identifies modules that contain "node_modules" in their path
4. Moves those modules to the vendors chunk using rspack's internal chunk graph API
5. **No `optimization.splitChunks` configuration required!**

#### Example Output

If you have a project with:

- `src/index.js` (your application code)
- `node_modules/lodash/lodash.js` (dependency)

After running the plugin, you'll get:

- `main.js` (3KB) - Contains only your application code
- `vendors.js` (549KB) - Contains all node_modules dependencies

#### Callback Support

The plugin supports a callback function that receives detailed information about moved modules and all chunks:

```javascript
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

    // Display all chunks and their modules
    console.log("🔍 All chunks and their modules:");
    chunks.forEach(([chunkName, modules]) => {
      console.log(`  Chunk: "${chunkName}" (${modules.length} modules)`);
      modules.forEach((module) => console.log(`    - ${module}`));
    });

    // Analyze chunks
    chunks.forEach(([chunkName, modules]) => {
      const nodeModulesCount = modules.filter((m) =>
        m.includes("node_modules")
      ).length;
      const appModulesCount = modules.filter(
        (m) => !m.includes("node_modules")
      ).length;
      console.log(
        `Chunk "${chunkName}": ${nodeModulesCount} node_modules, ${appModulesCount} app modules`
      );
    });

    // Create new chunks or modify existing ones
    addNewChunk("custom-chunk", movedModules.slice(0, 2));
    removeModuleFromChunk("vendors", movedModules.slice(0, 1));

    next(); // Resume execution
  },
});
```

#### Callback Parameters

- `movedModules`: Array of module paths that were moved to the vendors chunk
- `chunks`: Array of `[chunkName, modulePaths[]]` tuples containing all chunks and their modules
- `next`: Function to call when you want to resume the plugin execution
- `addNewChunk`: Function to create a new chunk with specified modules
- `removeModuleFromChunk`: Function to remove modules from a chunk

The callback is called automatically when modules are moved to the vendors chunk, providing you with real-time information about which modules were processed and full control over chunk management.

#### Technical Details

The plugin uses rspack's `CompilationOptimizeChunkModules` hook to:

- Create a new named chunk
- Iterate through all built modules
- Check module identifiers for "node_modules" paths
- Use `disconnect_chunk_and_module` and `connect_chunk_and_module` to move modules
- Return `Some(true)` to indicate changes were made
- Output callback data that is captured by the JavaScript wrapper

The JavaScript wrapper automatically handles the callback functionality by:

- Overriding `console.error` to capture plugin output
- Parsing the callback data from the plugin
- Calling the provided callback function with the moved modules array
- Restoring the original `console.error` after compilation

This approach gives you full control over chunk splitting without relying on rspack's built-in optimization features.

## Development

### Prerequisites

- Node.js 18+
- Rust toolchain
- pnpm

### Setup

```bash
pnpm install
```

### Build

```bash
pnpm build
```

### Test

```bash
cd examples/use-plugin
node build.js
```

## Project Structure

```
rspack-binding-demo/
├── crates/binding/          # Rust plugin implementation
│   ├── src/
│   │   ├── lib.rs          # Plugin registration
│   │   └── plugin.rs       # MyBannerPlugin implementation
│   └── Cargo.toml
├── lib/                     # JavaScript wrapper
│   ├── index.js            # Plugin wrapper with callback support
│   └── index.d.ts          # TypeScript definitions
├── examples/use-plugin/     # Usage example
│   ├── build.js            # Example configuration with callback
│   ├── src/index.js        # Example entry point
│   └── dist/               # Generated output
└── package.json
```

## License

MIT
