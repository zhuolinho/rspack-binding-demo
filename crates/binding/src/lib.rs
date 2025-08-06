mod plugin;

use napi::bindgen_prelude::*;
use rspack_binding_builder_macros::register_plugin;
use rspack_core::BoxPlugin;

#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

// Export a plugin named `MyBannerPlugin`.
//
// The plugin needs to be wrapped with `require('@rspack/core').experiments.createNativePlugin`
// to be used in the host.
//
// Check out `lib/index.js` for more details.
//
// `register_plugin` is a macro that registers a plugin.
//
// The first argument to `register_plugin` is the name of the plugin.
// The second argument to `register_plugin` is a resolver function that is called with `napi::Env` and the options returned from the resolver function from JS side.
//
// The resolver function should return a `BoxPlugin` instance.
register_plugin!("MyBannerPlugin", |_env: Env, options: Unknown<'_>| {
  // Parse options as an object
  let options_obj = options.coerce_to_object()?;

  // Get chunk_name from options
  let chunk_name = if let Ok(name) = options_obj.get_named_property::<String>("chunkName") {
    name
  } else {
    "vendors".to_string()
  };

  Ok(Box::new(plugin::MyBannerPlugin::new(chunk_name)) as BoxPlugin)
});

// Export the add_new_chunk function
#[napi]
pub fn add_new_chunk(chunk_name: String, module_paths: Vec<String>) -> bool {
  plugin::add_new_chunk(chunk_name, module_paths)
}

// Export the remove_module_from_chunk function
#[napi]
pub fn remove_module_from_chunk(chunk_name: String, module_paths: Vec<String>) -> bool {
  plugin::remove_module_from_chunk(chunk_name, module_paths)
}
