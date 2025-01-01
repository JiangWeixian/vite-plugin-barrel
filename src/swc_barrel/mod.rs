use std::collections::HashMap;

use serde::Deserialize;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::Fold;

fn collect_idents_in_object_pat(props: &[ObjectPatProp]) -> Vec<String> {
    let mut ids = Vec::new();
    for prop in props {
        match prop {
            ObjectPatProp::KeyValue(KeyValuePatProp { key, value }) => {
                if let PropName::Ident(ident) = key {
                    ids.push(ident.sym.to_string());
                }
                match &**value {
                    Pat::Ident(ident) => {
                        ids.push(ident.sym.to_string());
                    }
                    Pat::Array(array) => {
                        ids.extend(collect_idents_in_array_pat(&array.elems));
                    }
                    Pat::Object(object) => {
                        ids.extend(collect_idents_in_object_pat(&object.props));
                    }
                    _ => {}
                }
            }
            ObjectPatProp::Assign(AssignPatProp { key, .. }) => {
                ids.push(key.to_string());
            }
            ObjectPatProp::Rest(RestPat { arg, .. }) => {
                if let Pat::Ident(ident) = &**arg {
                    ids.push(ident.sym.to_string());
                }
            }
        }
    }
    ids
}

fn collect_idents_in_array_pat(elems: &[Option<Pat>]) -> Vec<String> {
    let mut ids = Vec::new();
    for elem in elems.iter().flatten() {
        match elem {
            Pat::Ident(ident) => {
                ids.push(ident.sym.to_string());
            }
            Pat::Array(array) => {
                ids.extend(collect_idents_in_array_pat(&array.elems));
            }
            Pat::Object(object) => {
                ids.extend(collect_idents_in_object_pat(&object.props));
            }
            Pat::Rest(rest) => {
                if let Pat::Ident(ident) = &*rest.arg {
                    ids.push(ident.sym.to_string());
                }
            }
            _ => {}
        }
    }
    ids
}

fn collect_idents_in_var_decls(decls: &[VarDeclarator]) -> Vec<String> {
    let mut ids = Vec::new();
    for decl in decls {
        match &decl.name {
            Pat::Ident(ident) => {
                ids.push(ident.sym.to_string());
            }
            Pat::Array(array) => {
                ids.extend(collect_idents_in_array_pat(&array.elems));
            }
            Pat::Object(object) => {
                ids.extend(collect_idents_in_object_pat(&object.props));
            }
            _ => {}
        }
    }
    ids
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub wildcard: bool,
}

struct Barrel {
    wildcard: bool,
}

impl Fold for Barrel {
    fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut local_idents = HashMap::new();
        for item in &items {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
                for spec in &import_decl.specifiers {
                    let src = import_decl.src.value.to_string();
                    match spec {
                        ImportSpecifier::Named(s) => {
                            local_idents.insert(
                                s.local.sym.to_string(),
                                (
                                    src.clone(),
                                    match &s.imported {
                                        Some(n) => match &n {
                                            ModuleExportName::Ident(n) => n.sym.to_string(),
                                            ModuleExportName::Str(n) => n.value.to_string(),
                                        },
                                        None => s.local.sym.to_string(),
                                    },
                                ),
                            );
                        }
                        ImportSpecifier::Namespace(s) => {
                            local_idents
                                .insert(s.local.sym.to_string(), (src.clone(), "*".to_string()));
                        }
                        ImportSpecifier::Default(s) => {
                            local_idents.insert(
                                s.local.sym.to_string(),
                                (src.clone(), "default".to_string()),
                            );
                        }
                    }
                }
            }
        }
        // The second pass to rebuild the module items.
        let mut new_items: Vec<ModuleItem> = vec![];

        // Exported meta information.
        let mut export_map = vec![];
        let mut export_wildcards = vec![];
        let mut is_barrel = true;
        for item in &items {
            match item {
                ModuleItem::ModuleDecl(decl) => {
                    match decl {
                        ModuleDecl::Import(_) => {}
                        // export { foo } from './foo';
                        ModuleDecl::ExportNamed(export_named) => {
                            for spec in &export_named.specifiers {
                                match spec {
                                    ExportSpecifier::Namespace(s) => {
                                        let name_str = match &s.name {
                                            ModuleExportName::Ident(n) => n.sym.to_string(),
                                            ModuleExportName::Str(n) => n.value.to_string(),
                                        };
                                        if let Some(src) = &export_named.src {
                                            export_map.push((
                                                name_str.clone(),
                                                src.value.to_string(),
                                                "*".to_string(),
                                            ));
                                        } else if self.wildcard {
                                            export_map.push((
                                                name_str.clone(),
                                                "".into(),
                                                "*".to_string(),
                                            ));
                                        } else {
                                            is_barrel = false;
                                            break;
                                        }
                                    }
                                    ExportSpecifier::Named(s) => {
                                        let orig_str = match &s.orig {
                                            ModuleExportName::Ident(n) => n.sym.to_string(),
                                            ModuleExportName::Str(n) => n.value.to_string(),
                                        };
                                        let name_str = match &s.exported {
                                            Some(n) => match &n {
                                                ModuleExportName::Ident(n) => n.sym.to_string(),
                                                ModuleExportName::Str(n) => n.value.to_string(),
                                            },
                                            None => orig_str.clone(),
                                        };

                                        if let Some(src) = &export_named.src {
                                            export_map.push((
                                                name_str.clone(),
                                                src.value.to_string(),
                                                orig_str.clone(),
                                            ));
                                        } else if let Some((src, orig)) =
                                            local_idents.get(&orig_str)
                                        {
                                            export_map.push((
                                                name_str.clone(),
                                                src.clone(),
                                                orig.clone(),
                                            ));
                                        } else if self.wildcard {
                                            export_map.push((
                                                name_str.clone(),
                                                "".into(),
                                                orig_str.clone(),
                                            ));
                                        } else {
                                            is_barrel = false;
                                            break;
                                        }
                                    }
                                    _ => {
                                        if !self.wildcard {
                                            is_barrel = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        // export * from "foo"
                        ModuleDecl::ExportAll(export_all) => {
                            export_wildcards.push(export_all.src.value.to_string());
                        }
                        // export function foo() {}
                        ModuleDecl::ExportDecl(export_decl) => {
                            // Export declarations are not allowed in barrel files.
                            if !self.wildcard {
                                is_barrel = false;
                                break;
                            }

                            match &export_decl.decl {
                                Decl::Class(class) => {
                                    export_map.push((
                                        class.ident.sym.to_string(),
                                        "".into(),
                                        "".into(),
                                    ));
                                }
                                Decl::Fn(func) => {
                                    export_map.push((
                                        func.ident.sym.to_string(),
                                        "".into(),
                                        "".into(),
                                    ));
                                }
                                Decl::Var(var) => {
                                    let ids = collect_idents_in_var_decls(&var.decls);
                                    for id in ids {
                                        export_map.push((id, "".into(), "".into()));
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            if !self.wildcard {
                                // Other expressions are not allowed in barrel files.
                                is_barrel = false;
                                break;
                            }
                        }
                    }
                }
                ModuleItem::Stmt(stmt) => match stmt {
                    Stmt::Expr(expr) => match &*expr.expr {
                        Expr::Lit(_) => {
                            new_items.push(item.clone());
                        }
                        _ => {
                            if !self.wildcard {
                                is_barrel = false;
                                break;
                            }
                        }
                    },
                    _ => {
                        if !self.wildcard {
                            is_barrel = false;
                            break;
                        }
                    }
                },
            }
        }

        // If the file is not a barrel file, we export nothing.
        if !is_barrel {
            new_items = vec![];
        } else {
            // Otherwise we export the meta information.
            new_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                span: DUMMY_SP,
                decl: Decl::Var(Box::new(VarDecl {
                    ctxt: Default::default(),
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(BindingIdent {
                            id: Ident {
                                span: DUMMY_SP,
                                sym: "__next_private_exports_map__".into(),
                                optional: false,
                                ctxt: Default::default(),
                            },
                            type_ann: None,
                        }),
                        init: Some(Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: serde_json::to_string(&export_map).unwrap().into(),
                            raw: None,
                        })))),
                        definite: false,
                    }],
                })),
            })));

            // Push "export *" statements for each wildcard export.
            for src in export_wildcards {
                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
                    span: DUMMY_SP,
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: format!("__barrel_optimize__?names=__PLACEHOLDER__&resourcePath={}", src)
                            .into(),
                        raw: None,
                    }),
                    with: None,
                    type_only: false,
                })));
            }
        }
        new_items
    }
}

pub fn barrel(config: &Config) -> impl Fold {
    Barrel {
        wildcard: config.wildcard,
    }
}
