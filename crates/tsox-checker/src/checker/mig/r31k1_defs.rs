use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};

static SOURCE_FILES_BY_ROOT_ID: OnceLock<RwLock<HashMap<u64, Arc<SourceFile>>>> = OnceLock::new();

pub(crate) fn register_diagnostic_source_files(files: &[Arc<SourceFile>]) { ::tsox_core::fntrace::enter("register_diagnostic_source_files"); 
    let mut map = SOURCE_FILES_BY_ROOT_ID
        .get_or_init(|| RwLock::new(HashMap::new()))
        .write()
        .unwrap();
    for file in files {
        map.insert(file.node.id(), Arc::clone(file));
    }
}

pub fn source_file_of_node(node: &Arc<Node>) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_of_node"); 
    let mut current = Arc::clone(node);
    loop {
        if current.kind == SyntaxKind::SourceFile {
            return SOURCE_FILES_BY_ROOT_ID
                .get_or_init(|| RwLock::new(HashMap::new()))
                .read()
                .unwrap()
                .get(&current.id())
                .cloned();
        }
        current = current.parent()?;
    }
}
