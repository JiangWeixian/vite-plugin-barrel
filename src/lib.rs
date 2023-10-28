pub mod swc_barrel;
pub mod swc_named_import_transform;

use swc_core::ecma::{
    ast::Program,
    transforms::testing::test,
    visit::{as_folder, FoldWith},
};
use serde_json::Value;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use swc_barrel::{barrel, Config};
use swc_named_import_transform::NamedImportTransformVisitor;

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
    let plugin_config: Value = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for relay"),
    )
    .expect("Should provide plugin config");
    let wildcard = plugin_config["wildcard"].as_bool().unwrap_or_default();
    let config = Config { wildcard };
    let mut barrel = barrel(&config);
    let program = program.fold_with(&mut as_folder(NamedImportTransformVisitor {}));
    program.fold_with(&mut barrel)
}

test!(
    Default::default(),
    |_| as_folder(NamedImportTransformVisitor {}),
    basic,
    // Input codes
    r#"import { Button, ALink } from "foo";"#,
    // Output codes after transformed with plugin
    r#"import path from "../path.js";"#
);
