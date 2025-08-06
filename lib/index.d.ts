import * as RspackCore from "@rspack/core";

/**
 * Options for MyBannerPlugin
 */
interface MyBannerPluginOptions {
  /**
   * The name of the vendors chunk to be created.
   */
  chunkName?: string;

  /**
   * Callback function that will be called when modules are moved to the vendors chunk.
   * @param movedModules Array of module paths that were moved to the vendors chunk
   * @param chunks Array of chunks with their modules, each item is [chunkName, modulePaths]
   * @param next Function to call when you want to resume the plugin execution
   * @param addNewChunk Function to create a new chunk with specified modules
   * @param removeModuleFromChunk Function to remove modules from a chunk
   */
  callback?: (
    movedModules: string[],
    chunks: [string, string[]][],
    next: () => void,
    addNewChunk: (chunkName: string, modulePaths: string[]) => boolean,
    removeModuleFromChunk: (chunkName: string, modulePaths: string[]) => boolean
  ) => void;
}

/**
 * MyBannerPlugin class that creates a vendors chunk and moves node_modules modules to it.
 */
declare class MyBannerPlugin {
  /**
   * Creates a new MyBannerPlugin instance.
   * @param options Either a string (chunk name) or an options object
   */
  constructor(options: string | MyBannerPluginOptions);
}

declare const core: typeof RspackCore & {
  MyBannerPlugin: typeof MyBannerPlugin;
};

export = core;
