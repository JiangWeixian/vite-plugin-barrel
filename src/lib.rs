pub mod swc_barrel;
pub mod swc_named_import_transform;

use serde::Deserialize;
use swc_core::ecma::{
    ast::Program,
    visit::{as_folder, FoldWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use swc_barrel::{barrel, Config as BarrelConfig};
use swc_named_import_transform::{named_import_transform, Config as NamedImportTransformConfig};

fn default_wildcard() -> bool {
    false
}

fn default_packages() -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
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
    let plugin_config = serde_json::from_str::<Config>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for swc_plugin_barrel"),
    )
    .expect("Should provide plugin config");
    // run two plugins
    let wildcard = plugin_config.wildcard;
    let packages = plugin_config.packages;
    println!("{:?}", packages);
    let barrel_config = BarrelConfig { wildcard };
    let mut barrel = barrel(&barrel_config);
    let named_config = NamedImportTransformConfig { packages };
    let named_import_transform = named_import_transform(named_config);
    // TODO: named-import-transform should run before barrel
    let program = program.fold_with(&mut as_folder(named_import_transform));
    program.fold_with(&mut barrel)
    // Run named plugin only
    // program.fold_with(&mut as_folder(named_import_transform))
}
