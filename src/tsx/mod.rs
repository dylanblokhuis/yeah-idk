use std::collections::HashMap;

use std::path::{Path, PathBuf};

use anyhow::{bail, Error};

use path_clean::PathClean;
use swc::atoms::js_word;
use swc_bundler::ModuleRecord;
use swc_bundler::{Bundler, Load, ModuleData};
use swc_bundler::{Config, Resolve};
use swc_common::comments::SingleThreadedComments;
use swc_common::{sync::Lrc, FileName, FilePathMapping, Globals, SourceMap};
use swc_common::{Mark, Span};
use swc_ecma_ast::*;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{parse_file_as_module, TsConfig};
use swc_ecma_transforms_base::{
    helpers::{inject_helpers, Helpers, HELPERS},
    resolver,
};
use swc_ecma_transforms_proposal::decorators;
use swc_ecma_transforms_react::react;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

pub fn compile_app() -> String {
    let globals = Globals::new();
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    // let external_modules = vec![];
    let mut bundler = Bundler::new(
        &globals,
        cm.clone(),
        PathLoader { cm: cm.clone() },
        NodeResolver,
        Config {
            require: true,
            disable_inliner: false,
            ..Default::default()
        },
        Box::new(Hook),
    );
    let mut entries = HashMap::default();
    entries.insert("main".to_string(), FileName::Real("./js/app.tsx".into()));
    let mut bundles = bundler.bundle(entries).expect("failed to bundle");
    let bundle = bundles.pop().unwrap();

    let mut buf = vec![];
    let wr = JsWriter::new(cm.clone(), "\n", &mut buf, None);

    let mut emitter = Emitter {
        cfg: swc_ecma_codegen::Config {
            minify: true,
            ..Default::default()
        },
        cm,
        comments: None,
        wr,
    };
    emitter.emit_module(&bundle.module).unwrap();

    return String::from_utf8_lossy(&buf).to_string();
}

struct PathLoader {
    cm: Lrc<SourceMap>,
}

impl Load for PathLoader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let tsx;
        let fm = match f {
            FileName::Real(path) => {
                tsx = path.to_string_lossy().ends_with(".tsx");
                self.cm.load_file(path)?
            }
            _ => unreachable!(),
        };

        let module = parse_file_as_module(
            &fm,
            Syntax::Typescript(TsConfig {
                decorators: true,
                tsx,
                ..Default::default()
            }),
            EsVersion::Es2020,
            None,
            &mut vec![],
        )
        .unwrap();

        let module = HELPERS.set(&Helpers::new(false), || {
            module
                .fold_with(&mut resolver(unresolved_mark, top_level_mark, false))
                .fold_with(&mut decorators(decorators::Config {
                    legacy: true,
                    emit_metadata: Default::default(),
                    use_define_for_class_fields: false,
                }))
                .fold_with(&mut react::<SingleThreadedComments>(
                    self.cm.clone(),
                    None,
                    Default::default(),
                    top_level_mark,
                ))
                .fold_with(&mut strip(top_level_mark))
                .fold_with(&mut inject_helpers())
        });

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}

struct Hook;

impl swc_bundler::Hook for Hook {
    fn get_import_meta_props(
        &self,
        span: Span,
        module_record: &ModuleRecord,
    ) -> Result<Vec<KeyValueProp>, Error> {
        let file_name = module_record.file_name.to_string();

        Ok(vec![
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("url"), span)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span,
                    raw: None,
                    value: file_name.into(),
                }))),
            },
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("main"), span)),
                value: Box::new(if module_record.is_entry {
                    Expr::Member(MemberExpr {
                        span,
                        obj: Box::new(Expr::MetaProp(MetaPropExpr {
                            span,
                            kind: MetaPropKind::ImportMeta,
                        })),
                        prop: MemberProp::Ident(Ident::new(js_word!("main"), span)),
                    })
                } else {
                    Expr::Lit(Lit::Bool(Bool { span, value: false }))
                }),
            },
        ])
    }
}

pub struct NodeResolver;

static EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "json", "node"];

impl NodeResolver {
    fn wrap(&self, path: PathBuf) -> Result<FileName, Error> {
        let path = path.clean();
        Ok(FileName::Real(path))
    }

    /// Resolve a path as a file. If `path` refers to a file, it is
    /// returned; otherwise the `path` + each extension is tried.
    fn resolve_as_file(&self, path: &Path) -> Result<PathBuf, Error> {
        // 1. If X is a file, load X as JavaScript text.
        if path.is_file() {
            return Ok(path.to_path_buf());
        }

        if let Some(name) = path.file_name() {
            let mut ext_path = path.to_path_buf();
            let name = name.to_string_lossy();
            for ext in EXTENSIONS {
                ext_path.set_file_name(format!("{}.{}", name, ext));
                if ext_path.is_file() {
                    return Ok(ext_path);
                }
            }

            // TypeScript-specific behavior: if the extension is ".js" or ".jsx",
            // try replacing it with ".ts" or ".tsx".
            ext_path.set_file_name(name.into_owned());
            let old_ext = path.extension().and_then(|ext| ext.to_str());

            if let Some(old_ext) = old_ext {
                let extensions = match old_ext {
                    // Note that the official compiler code always tries ".ts" before
                    // ".tsx" even if the original extension was ".jsx".
                    "js" => ["ts", "tsx"].as_slice(),
                    "jsx" => ["ts", "tsx"].as_slice(),
                    "mjs" => ["mts"].as_slice(),
                    "cjs" => ["cts"].as_slice(),
                    _ => [].as_slice(),
                };

                for ext in extensions {
                    ext_path.set_extension(ext);

                    if ext_path.is_file() {
                        return Ok(ext_path);
                    }
                }
            }
        }

        bail!("file not found: {}", path.display())
    }

    /// Resolve a path as a directory, using the "main" key from a
    /// package.json file if it exists, or resolving to the
    /// index.EXT file if it exists.
    fn resolve_as_directory(&self, path: &Path) -> Result<PathBuf, Error> {
        // 1. If X/package.json is a file, use it.
        let pkg_path = path.join("package.json");
        if pkg_path.is_file() {
            let main = self.resolve_package_main(&pkg_path);
            if main.is_ok() {
                return main;
            }
        }

        // 2. LOAD_INDEX(X)
        self.resolve_index(path)
    }

    /// Resolve using the package.json "main" key.
    fn resolve_package_main(&self, _: &Path) -> Result<PathBuf, Error> {
        bail!("package.json is not supported")
    }

    /// Resolve a directory to its index.EXT.
    fn resolve_index(&self, path: &Path) -> Result<PathBuf, Error> {
        // 1. If X/index.js is a file, load X/index.js as JavaScript text.
        // 2. If X/index.json is a file, parse X/index.json to a JavaScript object.
        // 3. If X/index.node is a file, load X/index.node as binary addon.
        for ext in EXTENSIONS {
            let ext_path = path.join(format!("index.{}", ext));
            if ext_path.is_file() {
                return Ok(ext_path);
            }
        }

        bail!("index not found: {}", path.display())
    }

    /// Resolve by walking up node_modules folders.
    fn resolve_node_modules(&self, base_dir: &Path, target: &str) -> Result<PathBuf, Error> {
        let node_modules = base_dir.join("node_modules");
        if node_modules.is_dir() {
            let path = node_modules.join(target);
            let result = self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path));
            if result.is_ok() {
                return result;
            }
        }

        match base_dir.parent() {
            Some(parent) => self.resolve_node_modules(parent, target),
            None => bail!("not found"),
        }
    }
}

impl Resolve for NodeResolver {
    fn resolve(&self, base: &FileName, target: &str) -> Result<FileName, Error> {
        let base = match base {
            FileName::Real(v) => v,
            _ => bail!("node-resolver supports only files"),
        };

        // Absolute path
        if target.starts_with('/') {
            let base_dir = &Path::new("/");

            let path = base_dir.join(target);
            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .and_then(|p| self.wrap(p));
        }

        let cwd = &Path::new(".");
        let mut base_dir = base.parent().unwrap_or(cwd);

        if target.starts_with("./") || target.starts_with("../") {
            let win_target;
            let target = if cfg!(target_os = "windows") {
                let t = if let Some(s) = target.strip_prefix("./") {
                    s
                } else {
                    base_dir = base_dir.parent().unwrap();
                    &target[3..]
                };
                win_target = t.replace('/', "\\");
                &*win_target
            } else {
                target
            };

            let path = base_dir.join(target);
            return self
                .resolve_as_file(&path)
                .or_else(|_| self.resolve_as_directory(&path))
                .and_then(|p| self.wrap(p));
        }

        self.resolve_node_modules(base_dir, target)
            .and_then(|p| self.wrap(p))
    }
}
