use rspack_core::{
  ApplyContext, Compilation, CompilationOptimizeChunkModules, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use std::fmt;
use std::sync::Mutex;
use std::sync::OnceLock;

// Global storage for callback data and pause state
static CALLBACK_DATA: OnceLock<Mutex<Option<Vec<String>>>> = OnceLock::new();
static CHUNKS_DATA: OnceLock<Mutex<Option<Vec<(String, Vec<String>)>>>> = OnceLock::new();
static IS_PAUSED: OnceLock<Mutex<bool>> = OnceLock::new();
static MODULES_TO_MOVE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static MODULES_MOVED: OnceLock<Mutex<bool>> = OnceLock::new();
static CHUNK_CREATION_REQUESTS: OnceLock<Mutex<Vec<(String, Vec<String>)>>> = OnceLock::new();
static MODULE_REMOVAL_REQUESTS: OnceLock<Mutex<Vec<(String, Vec<String>)>>> = OnceLock::new();

/// A plugin that creates a vendors chunk and moves node_modules modules to it.
#[plugin]
pub struct MyBannerPlugin {
  chunk_name: String,
}

impl fmt::Debug for MyBannerPlugin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MyBannerPlugin")
      .field("chunk_name", &self.chunk_name)
      .finish()
  }
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

  // Check if we're paused
  let pause_storage = IS_PAUSED.get_or_init(|| Mutex::new(false));
  if *pause_storage.lock().unwrap() {
    eprintln!("MyBannerPlugin: Still paused, skipping module movement");
    return Ok(Some(true));
  }

  // Check if we have stored modules to move (resumed execution)
  let modules_storage = MODULES_TO_MOVE.get_or_init(|| Mutex::new(Vec::new()));
  if let Ok(mut stored_modules) = modules_storage.lock() {
    if !stored_modules.is_empty() {
      eprintln!("MyBannerPlugin: Moving stored modules to vendors chunk...");

      // Get the modules to move
      let modules_to_move: Vec<String> = stored_modules.drain(..).collect();

      // First, collect all the module identifiers and their current chunks
      let mut module_operations = Vec::new();

      for module_path in modules_to_move {
        // Find the module identifier by path
        for module_identifier in compilation.built_modules() {
          if module_identifier.to_string() == module_path {
            // Get current chunks for this module
            let current_chunks: Vec<_> = compilation
              .chunk_graph
              .get_module_chunks(*module_identifier)
              .iter()
              .cloned()
              .collect();

            module_operations.push((*module_identifier, current_chunks));
            break;
          }
        }
      }

      // Now perform the operations without borrowing conflicts
      for (module_identifier, current_chunks) in module_operations {
        // Remove module from all current chunks
        for chunk_ukey in current_chunks {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&chunk_ukey, module_identifier);
          eprintln!("MyBannerPlugin: Removed module from chunk {:?}", chunk_ukey);
        }

        // Add module to vendors chunk
        compilation
          .chunk_graph
          .connect_chunk_and_module(vendors_chunk_ukey, module_identifier);
        eprintln!("MyBannerPlugin: Added module to vendors chunk");
      }

      // Force the vendors chunk to be included in the output
      eprintln!("MyBannerPlugin: Vendors chunk should be emitted with modules");

      // Set modules moved flag
      let modules_moved_storage = MODULES_MOVED.get_or_init(|| Mutex::new(false));
      *modules_moved_storage.lock().unwrap() = true;
      eprintln!("MyBannerPlugin: Modules moved flag set to true");

      // Process chunk creation requests after module movement
      let requests_storage = CHUNK_CREATION_REQUESTS.get_or_init(|| Mutex::new(Vec::new()));
      if let Ok(mut requests) = requests_storage.lock() {
        eprintln!(
          "MyBannerPlugin: Checking for chunk creation requests, count: {}",
          requests.len()
        );
        while let Some((chunk_name, module_paths)) = requests.pop() {
          eprintln!(
            "MyBannerPlugin: Processing chunk creation request for '{}'",
            chunk_name
          );

          // Create the new chunk
          let (new_chunk_ukey, new_chunk_created) = Compilation::add_named_chunk(
            chunk_name.clone(),
            &mut compilation.chunk_by_ukey,
            &mut compilation.named_chunks,
          );

          if new_chunk_created {
            compilation.chunk_graph.add_chunk(new_chunk_ukey);
            eprintln!("MyBannerPlugin: Created new chunk '{}'", chunk_name);
          }

          // Add modules to the new chunk
          for module_path in module_paths {
            // Find the module identifier by path
            for module_identifier in compilation.built_modules() {
              if module_identifier.to_string() == module_path {
                compilation
                  .chunk_graph
                  .connect_chunk_and_module(new_chunk_ukey, *module_identifier);
                eprintln!(
                  "MyBannerPlugin: Added module '{}' to chunk '{}'",
                  module_path, chunk_name
                );
                break;
              }
            }
          }
        }
      } else {
        eprintln!("MyBannerPlugin: Failed to lock chunk creation requests storage");
      }

      // Process module removal requests after module movement
      let removal_requests_storage = MODULE_REMOVAL_REQUESTS.get_or_init(|| Mutex::new(Vec::new()));
      if let Ok(mut removal_requests) = removal_requests_storage.lock() {
        eprintln!(
          "MyBannerPlugin: Checking for module removal requests, count: {}",
          removal_requests.len()
        );
        while let Some((chunk_name, module_paths)) = removal_requests.pop() {
          eprintln!(
            "MyBannerPlugin: Processing module removal request for chunk '{}'",
            chunk_name
          );

          // Find the chunk by name
          if let Some(chunk_ukey) = compilation.named_chunks.get(&chunk_name) {
            // Remove modules from the chunk
            for module_path in module_paths {
              // Find the module identifier by path
              for module_identifier in compilation.built_modules() {
                if module_identifier.to_string() == module_path {
                  compilation
                    .chunk_graph
                    .disconnect_chunk_and_module(chunk_ukey, *module_identifier);
                  eprintln!(
                    "MyBannerPlugin: Removed module '{}' from chunk '{}'",
                    module_path, chunk_name
                  );
                  break;
                }
              }
            }
          } else {
            eprintln!(
              "MyBannerPlugin: Chunk '{}' not found for module removal",
              chunk_name
            );
          }
        }
      } else {
        eprintln!("MyBannerPlugin: Failed to lock module removal requests storage");
      }

      return Ok(Some(true));
    }
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

  // If we have modules to move, pause and store them
  if !modules_to_move.is_empty() {
    // Collect chunks information
    let mut chunks_info = Vec::new();

    // Get all chunks from compilation
    for (chunk_name, chunk_ukey) in &compilation.named_chunks {
      let mut chunk_modules = Vec::new();

      // Get all modules in this chunk
      for module_identifier in compilation.built_modules() {
        let module_chunks = compilation
          .chunk_graph
          .get_module_chunks(*module_identifier);

        if module_chunks.contains(chunk_ukey) {
          chunk_modules.push(module_identifier.to_string());
        }
      }

      chunks_info.push((chunk_name.clone(), chunk_modules));
    }

    // Also include chunks without names (like main chunk)
    for chunk_ukey in compilation.chunk_by_ukey.keys() {
      let mut found_named = false;
      for (_, named_chunk_ukey) in &compilation.named_chunks {
        if chunk_ukey == named_chunk_ukey {
          found_named = true;
          break;
        }
      }

      if !found_named {
        let mut chunk_modules = Vec::new();
        for module_identifier in compilation.built_modules() {
          let module_chunks = compilation
            .chunk_graph
            .get_module_chunks(*module_identifier);

          if module_chunks.contains(chunk_ukey) {
            chunk_modules.push(module_identifier.to_string());
          }
        }

        // Use chunk ukey as name for unnamed chunks
        chunks_info.push((format!("{:?}", chunk_ukey), chunk_modules));
      }
    }

    // Store chunks information
    let chunks_storage = CHUNKS_DATA.get_or_init(|| Mutex::new(None));
    *chunks_storage.lock().unwrap() = Some(chunks_info);

    // Store the modules that will be moved
    let callback_storage = CALLBACK_DATA.get_or_init(|| Mutex::new(None));
    let module_paths: Vec<String> = modules_to_move
      .iter()
      .map(|(id, _)| id.to_string())
      .collect();
    *callback_storage.lock().unwrap() = Some(module_paths.clone());

    // Store modules to move for later execution
    let modules_storage = MODULES_TO_MOVE.get_or_init(|| Mutex::new(Vec::new()));
    *modules_storage.lock().unwrap() = module_paths;

    // Set pause flag
    *pause_storage.lock().unwrap() = true;

    eprintln!("MyBannerPlugin: Pausing execution, waiting for next() call...");

    // Return early to pause execution
    return Ok(Some(true));
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

// Function to get callback data (will be called from JavaScript)
#[napi]
pub fn get_callback_data() -> Option<Vec<String>> {
  if let Some(callback_storage) = CALLBACK_DATA.get() {
    if let Ok(mut data) = callback_storage.lock() {
      data.take() // Take the data and leave None
    } else {
      None
    }
  } else {
    None
  }
}

// Function to get chunks data (will be called from JavaScript)
#[napi]
pub fn get_chunks_data() -> Option<Vec<(String, Vec<String>)>> {
  if let Some(chunks_storage) = CHUNKS_DATA.get() {
    if let Ok(mut data) = chunks_storage.lock() {
      data.take() // Take the data and leave None
    } else {
      None
    }
  } else {
    None
  }
}

// Function to resume execution (will be called from JavaScript)
#[napi]
pub fn resume_execution() -> bool {
  // Clear the pause flag
  let pause_storage = IS_PAUSED.get_or_init(|| Mutex::new(false));
  *pause_storage.lock().unwrap() = false;

  eprintln!("MyBannerPlugin: Resume flag cleared, execution will continue on next hook call");
  true
}

// Function to add a new chunk and connect modules to it
#[napi]
pub fn add_new_chunk(chunk_name: String, module_paths: Vec<String>) -> bool {
  eprintln!(
    "MyBannerPlugin: Adding new chunk '{}' with {} modules",
    chunk_name,
    module_paths.len()
  );

  // Check if modules have been moved
  let modules_moved_storage = MODULES_MOVED.get_or_init(|| Mutex::new(false));
  let modules_moved = *modules_moved_storage.lock().unwrap();

  if !modules_moved {
    eprintln!("MyBannerPlugin: Modules not yet moved, storing chunk creation request for later");
    // Store the chunk creation request in a global storage
    let requests_storage = CHUNK_CREATION_REQUESTS.get_or_init(|| Mutex::new(Vec::new()));

    if let Ok(mut requests) = requests_storage.lock() {
      requests.push((chunk_name, module_paths));
      eprintln!("MyBannerPlugin: Stored chunk creation request");
    }
  } else {
    eprintln!(
      "MyBannerPlugin: Modules already moved, chunk creation request will be processed immediately"
    );
  }

  true
}

// Function to remove modules from a chunk
#[napi]
pub fn remove_module_from_chunk(chunk_name: String, module_paths: Vec<String>) -> bool {
  eprintln!(
    "MyBannerPlugin: Removing {} modules from chunk '{}'",
    module_paths.len(),
    chunk_name
  );

  // Check if modules have been moved
  let modules_moved_storage = MODULES_MOVED.get_or_init(|| Mutex::new(false));
  let modules_moved = *modules_moved_storage.lock().unwrap();

  if !modules_moved {
    eprintln!("MyBannerPlugin: Modules not yet moved, storing module removal request for later");
    // Store the module removal request in a global storage
    let requests_storage = MODULE_REMOVAL_REQUESTS.get_or_init(|| Mutex::new(Vec::new()));

    if let Ok(mut requests) = requests_storage.lock() {
      requests.push((chunk_name, module_paths));
      eprintln!("MyBannerPlugin: Stored module removal request");
    }
  } else {
    eprintln!(
      "MyBannerPlugin: Modules already moved, module removal request will be processed immediately"
    );
  }

  true
}
