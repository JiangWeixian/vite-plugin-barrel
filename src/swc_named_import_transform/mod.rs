use std::collections::HashSet;

use serde::Deserialize;
use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::testing::test;
use swc_core::ecma::visit::{as_folder, Fold, noop_fold_type, VisitMut};
use swc_core::common::DUMMY_SP;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub packages: Vec<String>
}

#[derive(Debug, Default)]
pub struct NamedImportTransformVisitor {
    pub packages: Vec<String>
}

impl VisitMut for NamedImportTransformVisitor {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
    fn visit_mut_import_decl(&mut self, import: &mut ImportDecl) {
        let src_value = import.src.value.clone();
        let mut specifier_names = HashSet::new();
        let mut skip_transform = false;
        if self.packages.iter().any(|p| src_value == *p) {

            for specifier in &import.specifiers {
                match specifier {
                    ImportSpecifier::Named(specifier) => {
                        // Add the import name as string to the set
                        if let Some(imported) = &specifier.imported {
                            match imported {
                                ModuleExportName::Ident(ident) => {
                                    specifier_names.insert(ident.sym.to_string());
                                }
                                ModuleExportName::Str(str_) => {
                                    specifier_names.insert(str_.value.to_string());
                                }
                            }
                        } else {
                            specifier_names.insert(specifier.local.sym.to_string());
                        }
                    }
                    ImportSpecifier::Default(_) => {
                        skip_transform = true;
                        break;
                    }
                    ImportSpecifier::Namespace(_) => {
                        skip_transform = true;
                        break;
                    }
                }
            }
            if !skip_transform {
                let mut names = specifier_names.into_iter().collect::<Vec<_>>();
                // Sort the names to make sure the order is consistent
                names.sort();

                let new_src = format!(
                    "__barrel_optimize__?names={}!=!{}",
                    names.join(","),
                    src_value
                );

                // Create a new import declaration, keep everything the same except the source
                import.src = Box::new(Str {
                    span: DUMMY_SP,
                    value: new_src.into(),
                    raw: None,
                });
            }
        }
    }
}

pub fn named_import_transform(config: Config) -> impl VisitMut {
    NamedImportTransformVisitor {
        packages: config.packages
    }
}

pub struct TransformImports {
    pub packages: Vec<String>
}

impl Fold for TransformImports {
    noop_fold_type!();

    fn fold_module(&mut self, mut module: Module) -> Module {
        let mut new_items: Vec<ModuleItem> = vec![];
        for item in module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
                    let src_value = decl.src.value.clone();
                    let mut skip_transform = false;
                    if self.packages.iter().any(|p| src_value == *p) {
                        for specifier in &decl.specifiers {
                            match specifier {
                                ImportSpecifier::Named(specifier) => {
                                    // Add the import name as string to the set
                                    if let Some(imported) = &specifier.imported {
                                        match imported {
                                            ModuleExportName::Ident(ident) => {
                                                let new_src = format!(
                                                    "__barrel_optimize__?names={}!=!{}",
                                                    ident.sym.to_string(),
                                                    src_value
                                                );
                                                let specifiers = ImportSpecifier::Named(specifier.clone());
                                                let import = ImportDecl {
                                                    span: DUMMY_SP,
                                                    src: Box::new(Str {
                                                        span: DUMMY_SP,
                                                        value: new_src.into(),
                                                        raw: None,
                                                    }),
                                                    type_only: false,
                                                    with: None,
                                                    specifiers: vec![specifiers]
                                                };
                                                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                                            }
                                            ModuleExportName::Str(str_) => {
                                                let new_src = format!(
                                                    "__barrel_optimize__?names={}!=!{}",
                                                    str_.value.to_string(),
                                                    src_value
                                                );
                                                let specifiers = ImportSpecifier::Named(specifier.clone());
                                                let import = ImportDecl {
                                                    span: DUMMY_SP,
                                                    src: Box::new(Str {
                                                        span: DUMMY_SP,
                                                        value: new_src.into(),
                                                        raw: None,
                                                    }),
                                                    type_only: false,
                                                    with: None,
                                                    specifiers: vec![specifiers]
                                                };
                                                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                                            }
                                        }
                                    } else {
                                        let new_src = format!(
                                            "__barrel_optimize__?names={}!=!{}",
                                            specifier.local.sym.to_string(),
                                            src_value
                                        );
                                        let specifiers = ImportSpecifier::Named(specifier.clone());
                                        let import = ImportDecl {
                                            span: DUMMY_SP,
                                            src: Box::new(Str {
                                                span: DUMMY_SP,
                                                value: new_src.into(),
                                                raw: None,
                                            }),
                                            type_only: false,
                                            with: None,
                                            specifiers: vec![specifiers]
                                        };
                                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                                    }
                                }
                                ImportSpecifier::Default(_) => {
                                    skip_transform = true;
                                    break;
                                }
                                ImportSpecifier::Namespace(_) => {
                                    skip_transform = true;
                                    break;
                                }
                            }
                        }
                    } else {
                        skip_transform = true
                    }
                    if skip_transform {
                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(decl)));
                    }
                }
                x => {
                    new_items.push(x);
                }
            }
        }
        module.body = new_items;
        module
    }
}

pub fn transform_imports(config: Config) -> impl Fold {
    TransformImports { packages: config.packages }
}

test!(
  Default::default(),
  |_| as_folder(NamedImportTransformVisitor { packages: vec!["foo".to_string()] }),
  basic,
  // Input codes
  r#"import { Button, ALink } from "foo";"#,
  // Output codes after transformed with plugin
  r#"import { Button, ALink } from "__barrel_optimize__?names=ALink,Button!=!foo";"#
);