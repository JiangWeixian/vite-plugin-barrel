use std::path::PathBuf;

use swc_core::ecma::transforms::testing::test_fixture;
use testing::fixture;

use swc_plugin_barrel::swc_barrel::{barrel, Config};
use swc_plugin_barrel::swc_named_import_transform::{transform_imports, Config as TransformImportsConfig};

#[fixture("tests/fixture/basic/**/input.ts")]
fn fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        Default::default(),
        &|_| {
            barrel(&Config { wildcard: false })
        },
        &input,
        &output,
        Default::default(),
    );
}

#[fixture("tests/fixture/transform-imports/**/input.ts")]
fn fixture_transform_imports(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    let package = String::from("foo");

    test_fixture(
        Default::default(),
        &|_| {
            transform_imports(TransformImportsConfig { packages: vec![package.clone()] })
        },
        &input,
        &output,
        Default::default(),
    );
}