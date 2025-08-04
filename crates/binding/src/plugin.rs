use rspack_core::{
  ApplyContext, Compilation, CompilationOptimizeChunkModules, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

/// A plugin that creates a vendors chunk and moves node_modules modules to it.
#[derive(Debug)]
#[plugin]
pub struct MyBannerPlugin {
  chunk_name: String,
}

impl MyBannerPlugin {
  pub fn new(chunk_name: String) -> Self {
    Self::new_inner(chunk_name)
  }
}

#[plugin_hook(CompilationOptimizeChunkModules for MyBannerPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONS, tracing = false)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  eprintln!(
    "MyBannerPlugin: Creating vendors chunk '{}'",
    self.chunk_name
  );

  // Create vendors chunk
  let (vendors_chunk_ukey, created) = Compilation::add_named_chunk(
    self.chunk_name.clone(),
    &mut compilation.chunk_by_ukey,
    &mut compilation.named_chunks,
  );

  if created {
    compilation.chunk_graph.add_chunk(vendors_chunk_ukey);
    eprintln!("MyBannerPlugin: Created new vendors chunk");
  }

  // Get all modules from compilation
  let modules = compilation.built_modules();

  // Collect modules that need to be moved to vendors chunk
  let mut modules_to_move = Vec::new();

  // Iterate through all modules to identify node_modules modules
  for module_identifier in modules {
    // Get module identifier to check if it's from node_modules
    let identifier_str = module_identifier.to_string();

    // Check if the module path contains node_modules
    if identifier_str.contains("node_modules") {
      eprintln!(
        "MyBannerPlugin: Found node_modules module: {}",
        identifier_str
      );

      // Get current chunks for this module and clone them to avoid borrow checker issues
      let current_chunks: Vec<_> = compilation
        .chunk_graph
        .get_module_chunks(*module_identifier)
        .iter()
        .cloned()
        .collect();

      // If module is not already in vendors chunk, mark it for moving
      if !current_chunks.contains(&vendors_chunk_ukey) {
        modules_to_move.push((*module_identifier, current_chunks));
      }
    }
  }

  // Now move the modules to vendors chunk
  for (module_identifier, current_chunks) in modules_to_move {
    let identifier_str = module_identifier.to_string();

    // Remove module from all current chunks
    for chunk_ukey in current_chunks {
      compilation
        .chunk_graph
        .disconnect_chunk_and_module(&chunk_ukey, module_identifier);
      eprintln!(
        "MyBannerPlugin: Removed module {} from chunk {:?}",
        identifier_str, chunk_ukey
      );
    }

    // Add module to vendors chunk
    compilation
      .chunk_graph
      .connect_chunk_and_module(vendors_chunk_ukey, module_identifier);
    eprintln!(
      "MyBannerPlugin: Added module {} to vendors chunk",
      identifier_str
    );
  }

  // Return true to indicate that we made changes
  Ok(Some(true))
}

impl Plugin for MyBannerPlugin {
  fn name(&self) -> &'static str {
    "MyBannerPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    Ok(())
  }
}
