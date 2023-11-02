pub mod swc_barrel;
pub mod swc_named_import_transform;

use std::path::PathBuf;
use serde::Deserialize;
use swc_core::ecma::{
    ast::Program,
    visit::{as_folder, FoldWith},
};
use swc_core::common::FileName;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata, metadata::TransformPluginMetadataContextKind};
use swc_barrel::{barrel, Config as BarrelConfig};
// use swc_named_import_transform::{named_import_transform, Config as NamedImportTransformConfig};
use swc_named_import_transform::{transform_imports, Config as NamedImportTransformConfig};

fn default_plugin_barrel() -> bool {
    false
}

fn default_wildcard() -> bool {
    false
}

fn default_packages() -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_plugin_barrel")]
    pub enable_plugin_barrel: bool,
    #[serde(default = "default_wildcard")]
    pub wildcard: bool,
    #[serde(default = "default_packages")]
    pub packages: Vec<String>
}

/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let filename = if let Some(filename) =
        metadata.get_context(&TransformPluginMetadataContextKind::Filename)
    {
        FileName::Real(PathBuf::from(filename))
    } else {
        FileName::Anon
    };
    println!("filename: {:?}", filename);
    let plugin_config = serde_json::from_str::<Config>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for swc_plugin_barrel"),
    )
    .expect("Should provide plugin config");
    // run two plugins
    let wildcard = plugin_config.wildcard;
    let packages = plugin_config.packages;
    let enable_plugin_barrel = plugin_config.enable_plugin_barrel;
    let barrel_config = BarrelConfig { wildcard };
    let mut barrel = barrel(&barrel_config);
    let named_config = NamedImportTransformConfig { packages };
    // TODO: split it into two plugins
    // let named_import_transform = named_import_transform(named_config);
    // let program = program.fold_with(&mut as_folder(named_import_transform));
    let mut named_import_transform = transform_imports(named_config);
    let program = program.fold_with(&mut named_import_transform);
    if enable_plugin_barrel == true {
        program.fold_with(&mut barrel)
    } else {
        program
    }
    // Run named plugin only
    // program.fold_with(&mut as_folder(named_import_transform))
}
