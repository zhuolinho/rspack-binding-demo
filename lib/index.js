process.env.RSPACK_BINDING = require("node:path").dirname(
  require.resolve("@rspack-template/binding")
);

const binding = require("@rspack-template/binding");

// Register the plugin `MyBannerPlugin` exported by `crates/binding/src/lib.rs`.
binding.registerMyBannerPlugin();

const core = require("@rspack/core");

/**
 * Creates a wrapper for the plugin `MyBannerPlugin` exported by `crates/binding/src/lib.rs`.
 *
 * Check out `crates/binding/src/lib.rs` for the original plugin definition.
 * This plugin is used in `examples/use-plugin/build.js`.
 *
 * @example
 * ```js
 * const MyBannerPlugin = require('@rspack-template/core').MyBannerPlugin;
 * ```
 *
 * `createNativePlugin` is a function that creates a wrapper for the plugin.
 *
 * The first argument to `createNativePlugin` is the name of the plugin.
 * The second argument to `createNativePlugin` is a resolver function that is called with the options passed to the plugin constructor.
 *
 * The resolver function should return the options that will be passed to the Rust plugin.
 */
const MyBannerPlugin = core.experiments.createNativePlugin(
  "MyBannerPlugin",
  (options) => {
    // If options is a string, treat it as chunkName
    if (typeof options === "string") {
      return {
        chunkName: options,
      };
    }

    // If options is an object, pass it through
    if (typeof options === "object" && options !== null) {
      return {
        chunkName: options.chunkName || "vendors",
      };
    }

    // Default fallback
    return {
      chunkName: "vendors",
    };
  }
);

// Create a wrapper that handles callback functionality
class MyBannerPluginWrapper {
  constructor(options) {
    this.options = options;
    this.callback = options.callback;

    // Create the actual plugin instance
    this.plugin = new MyBannerPlugin(options);
  }

  apply(compiler) {
    // Apply the actual plugin
    this.plugin.apply(compiler);

    // Check for callback data after compilation
    if (this.callback && typeof this.callback === "function") {
      compiler.hooks.done.tap("MyBannerPluginWrapper", () => {
        // Use setTimeout to ensure callback data is available
        // setTimeout(() => {
        try {
          const callbackData = binding.getCallbackData();
          if (callbackData) {
            this.callback(callbackData);
          }
        } catch (e) {
          // Ignore errors
        }
        // }, 200);
      });
    }
  }
}

// Export everything from core, plus our custom plugin
Object.assign(core, {
  MyBannerPlugin: MyBannerPluginWrapper,
});

module.exports = core;
