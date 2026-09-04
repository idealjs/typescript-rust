use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local = manifest_dir.join("testdata/baselines/local");
    let reference = manifest_dir.join("testdata/baselines/reference");

    if !local.exists() {
        eprintln!(
            "No local baselines found at {}. Run tests first.",
            local.display()
        );
        std::process::exit(1);
    }

    let count = copy_dir_recursive(&local, &reference);
    println!(
        "Accepted {} baseline files from {} → {}",
        count,
        local.display(),
        reference.display()
    );
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let rel = path.strip_prefix(src).unwrap();
                let new_dst = dst.join(rel);
                count += copy_dir_recursive(&path, &new_dst);
            } else {
                if let Some(parent) = dst.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let target = dst.join(path.file_name().unwrap());
                if fs::copy(&path, &target).is_ok() {
                    count += 1;
                }
            }
        }
    }
    count
}
