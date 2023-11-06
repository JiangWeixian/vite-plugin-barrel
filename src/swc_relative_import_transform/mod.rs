use std::path::Path;

use serde::Deserialize;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{Fold, noop_fold_type};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub enable: bool,
}

#[derive(Debug, Default)]
pub struct RelativeImportTransform {
    pub enable: bool,
}

impl Fold for RelativeImportTransform {
    noop_fold_type!();

    fn fold_module(&mut self, mut module: Module) -> Module {
        let mut new_items: Vec<ModuleItem> = vec![];
        for item in module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
                    let src_value = decl.src.value.clone();
                    let mut skip_transform = false;

                    let src_ref = &src_value.to_string();
                    let ext = Path::new(src_ref).extension();

                    // transform relative imports and without ext
                    // import { x } from "./foo" not import { x } from "./foo.css"
                    if src_value.starts_with(".") && !ext.is_some() && self.enable {
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
                                                let specifiers =
                                                    ImportSpecifier::Named(specifier.clone());
                                                let import = ImportDecl {
                                                    span: DUMMY_SP,
                                                    src: Box::new(Str {
                                                        span: DUMMY_SP,
                                                        value: new_src.into(),
                                                        raw: None,
                                                    }),
                                                    type_only: false,
                                                    with: None,
                                                    specifiers: vec![specifiers],
                                                };
                                                new_items.push(ModuleItem::ModuleDecl(
                                                    ModuleDecl::Import(import),
                                                ))
                                            }
                                            ModuleExportName::Str(str_) => {
                                                let new_src = format!(
                                                    "__barrel_optimize__?names={}!=!{}",
                                                    str_.value.to_string(),
                                                    src_value
                                                );
                                                let specifiers =
                                                    ImportSpecifier::Named(specifier.clone());
                                                let import = ImportDecl {
                                                    span: DUMMY_SP,
                                                    src: Box::new(Str {
                                                        span: DUMMY_SP,
                                                        value: new_src.into(),
                                                        raw: None,
                                                    }),
                                                    type_only: false,
                                                    with: None,
                                                    specifiers: vec![specifiers],
                                                };
                                                new_items.push(ModuleItem::ModuleDecl(
                                                    ModuleDecl::Import(import),
                                                ))
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
                                            specifiers: vec![specifiers],
                                        };
                                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(
                                            import,
                                        )))
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


pub fn relative_import_transform(config: &Config) -> impl Fold {
    RelativeImportTransform { enable: config.enable }
}
