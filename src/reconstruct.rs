use std::path::Path;
use crate::des::SourceMapModel;

pub struct ProjectReconstructor<'a> {
    files: Vec<(&'a str, Option<&'a str>)>
}

impl<'a> ProjectReconstructor<'a> {
    pub fn new(source_map: &'a SourceMapModel) -> Self {
        let mut v = Vec::new();
        let contents = source_map.sources_content.as_ref().unwrap();
        for (i, name) in source_map.names.iter().enumerate() {
            let content = contents[i].as_deref();
            v.push((name.as_str(), content))
        }
        
        Self {
            files: v
        }
    }
    
    pub fn extract_to(&self, path: &Path) {
        todo!();
    }
}

