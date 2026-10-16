use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ROOT_PATH: OnceLock<String> = OnceLock::new();

pub fn root_path() -> &'static str { crate::fntrace::enter("root_path"); 
    ROOT_PATH.get_or_init(|| {
        let filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        if !filename.is_absolute() {
            panic!("{} is not an absolute path", filename.display());
        }

        let root = Path::new(&filename)
            .ancestors()
            .last()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));

        let mut dir: &Path = &filename;
        loop {
            if dir.join("Cargo.toml").exists() && is_workspace_root(dir) {
                return dir.to_string_lossy().into_owned();
            }
            if dir == root {
                break;
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }

        panic!("could not find Cargo.toml above {}", filename.display())
    })
}

fn is_workspace_root(dir: &Path) -> bool { crate::fntrace::enter("is_workspace_root"); 
    std::fs::read_to_string(dir.join("Cargo.toml"))
        .map(|content| content.contains("[workspace]"))
        .unwrap_or(false)
}

pub fn RootPath() -> &'static str { crate::fntrace::enter("RootPath"); 
    root_path()
}
