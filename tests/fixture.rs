use std::path::PathBuf;

use swc_core::ecma::transforms::testing::test_fixture;
use testing::fixture;

use swc_plugin_barrel::swc_barrel::{barrel, Config};

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

#[fixture("tests/fixture/wildcard/**/input.ts")]
fn fixture_wildcard(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        Default::default(),
        &|_| {
            barrel(&Config { wildcard: true })
        },
        &input,
        &output,
        Default::default(),
    );
}