use serde::Deserialize;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{noop_fold_type, Fold};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub packages: Vec<String>,
}

#[derive(Debug, Default)]
pub struct NamedImportTransform {
    pub packages: Vec<String>,
}

impl Fold for NamedImportTransform {
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
                                                    "__barrel_optimize__?names={}&resourcePath={}",
                                                    ident.sym,
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
                                                    "__barrel_optimize__?names={}&resourcePath={}",
                                                    str_.value,
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
                                            "__barrel_optimize__?names={}&resourcePath={}",
                                            specifier.local.sym,
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

pub fn named_import_transform(config: Config) -> impl Fold {
    NamedImportTransform {
        packages: config.packages,
    }
}
