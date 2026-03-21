use crate::des::SourceMapModel;
use std::fs;
use std::path::{Component, Path, PathBuf};

struct Module<'a> {
    files: Vec<(&'a str, Option<&'a str>)>,
    name: Option<String>,
}

impl<'a> Module<'a> {
    pub fn normalized_path_iterator(&self) -> impl Iterator<Item = (&'a str, &Option<&'a str>)> {
        self.files
            .iter()
            .map(|(p, src)| (p.strip_prefix("webpack://").unwrap_or(p), src))
    }

    pub fn max_base_nestings(&self) -> usize {
        /*
        ./CCLibraries/../../../src/drivers/Driver.ts
        0 1           2  3  4  5   6       7

        0: []
        1: [CCLibraries]
        2: []
        3: [ParentDir]
        4: [ParentDir, ParentDir]
        5: [ParentDir, ParentDir, src]
        6: [ParentDir, ParentDir, src, drivers]
        7: [ParentDir, ParentDir, src, drivers, Driver.ts]
        */
        let mut nestings = 0;

        for (p, _) in self.normalized_path_iterator() {
            let path = Path::new(p);
            let mut normals = 0usize;
            let mut escapes = 0usize;

            for piece in path.components() {
                match piece {
                    Component::Normal(_) => normals += 1,
                    Component::ParentDir => {
                        if normals > 0 {
                            normals -= 1;
                        } else {
                            escapes += 1;
                        }
                    }
                    _ => {}
                }
            }

            nestings = nestings.max(escapes);
        }

        nestings
    }
}

pub struct ProjectReconstructor<'a> {
    modules: Vec<Module<'a>>,
}

impl<'a> ProjectReconstructor<'a> {
    pub fn new(source_maps: &[&'a SourceMapModel]) -> Self {
        let mut modules = Vec::new();

        for module in source_maps {
            let mut files = Vec::new();
            let contents = module.sources_content.as_ref().unwrap();
            println!("Loading sourcemap: {}", module.file.as_deref().unwrap());

            for (i, name) in module.sources.iter().enumerate() {
                let content = contents[i].as_deref();
                files.push((name.as_str(), content))
            }

            modules.push(Module {
                files,
                name: module.file.clone(),
            })
        }

        Self { modules }
    }

    fn max_base_nestings(&self) -> usize {
        self.modules
            .iter()
            .map(|m| m.max_base_nestings())
            .max()
            .unwrap()
    }

    pub fn dump_size(&self) -> (usize, usize) {
        let fcount = self.modules.iter().map(|m| m.files.len()).sum();
        let fsize = self
            .modules
            .iter()
            .map(|m| {
                m.files
                    .iter()
                    .filter_map(|f| f.1)
                    .map(|c| c.len())
                    .sum::<usize>()
            })
            .sum();

        (fcount, fsize)
    }

    pub fn extract_to(&self, path: &Path) {
        let max_nestings = self.max_base_nestings();

        let base: PathBuf = (0..=max_nestings)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("/")
            .into();

        let absolute_base = &path.join(base);
        fs::create_dir_all(&absolute_base);

        for module in &self.modules {
            for (fpath, source) in module.normalized_path_iterator() {
                if let Some(source) = source {
                    let full_path = absolute_base.join(fpath);
                    fs::create_dir_all(full_path.parent().unwrap());
                    fs::write(full_path, source);
                }
            }
        }
    }
}
