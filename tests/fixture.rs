use std::path::PathBuf;

use swc_core::ecma::transforms::testing::test_fixture;
use testing::fixture;

use swc_plugin_barrel::swc_barrel::{barrel, Config};
use swc_plugin_barrel::swc_named_import_transform::{
    named_import_transform, Config as NamedImportTransformConfig,
};
use swc_plugin_barrel::swc_relative_import_transform::{
    relative_import_transform, Config as RelativeTransformImportsConfig,
};

#[fixture("tests/fixture/basic/**/input.ts")]
fn fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        Default::default(),
        &|_| barrel(&Config { wildcard: false }),
        &input,
        &output,
        Default::default(),
    );
}

#[fixture("tests/fixture/named-imports/**/input.ts")]
fn fixture_transform_named_imports(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    let package = String::from("foo");

    test_fixture(
        Default::default(),
        &|_| {
            named_import_transform(NamedImportTransformConfig {
                packages: vec![package.clone()],
            })
        },
        &input,
        &output,
        Default::default(),
    );
}

#[fixture("tests/fixture/relative-imports/1/input.ts")]
fn fixture_transform_relative_imports(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        Default::default(),
        &|_| {
            relative_import_transform(&RelativeTransformImportsConfig {
                enable: true,
            })
        },
        &input,
        &output,
        Default::default(),
    );
}

#[fixture("tests/fixture/relative-imports/2/input.ts")]
fn fixture_disable_transform_relative_imports(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        Default::default(),
        &|_| {
            relative_import_transform(&RelativeTransformImportsConfig {
                enable: false,
            })
        },
        &input,
        &output,
        Default::default(),
    );
}
