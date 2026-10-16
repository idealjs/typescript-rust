use std::path::Path;
use std::sync::Arc;

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::InMemoryFS;

pub fn mount_test_lib_fixtures(fs: &Arc<InMemoryFS>) {
    fn walk(fs: &Arc<InMemoryFS>, src: &Path, mount: &str) {
        let Ok(entries) = std::fs::read_dir(src) else {
            return;
        };
        fs.insert_dir(mount);
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let mount_path = format!("{mount}/{name}");
            if p.is_dir() {
                walk(fs, &p, &mount_path);
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                fs.insert_file(&mount_path, &text);
            }
        }
    }
    let root = Path::new("tests/corpus/testdata/tests/lib");
    walk(fs, &root, "/.lib");
}

pub fn unit_abs_path(unit_name: &str) -> String {
    if tsox_core::tspath::is_rooted_disk_path(unit_name) {
        tsox_core::tspath::normalize_path(unit_name)
    } else {
        tsox_core::tspath::normalize_path(&format!("/proj/{}", unit_name))
    }
}

pub fn is_config_unit(unit_name: &str) -> bool {
    let base = unit_name.rsplit(['/', '\\']).next().unwrap_or(unit_name);
    let lower = base.to_ascii_lowercase();
    lower == "tsconfig.json" || lower == "jsconfig.json"
}

pub fn detect_tsconfig(
    units: &[crate::case_parser::TestUnit],
) -> Option<(ParsedCommandLine, String)> {
    let config_unit = units.iter().find(|u| is_config_unit(&u.name))?;
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    for unit in units {
        let abs = unit_abs_path(&unit.name);
        let mut parent = tsox_core::tspath::get_directory_path(&abs);
        while !parent.is_empty() {
            fs.insert_dir(&parent);
            let next = tsox_core::tspath::get_directory_path(&parent);
            if next == parent {
                break;
            }
            parent = next;
        }
        fs.insert_file(&abs, &unit.content);
    }

    let cwd = if units.iter().any(|u| tsox_core::tspath::is_rooted_disk_path(&u.name)) {
        "/.src"
    } else {
        "/proj"
    };
    let config_path = unit_abs_path(&config_unit.name);
    Some((
        tsox_tsoptions::tsoptions::get_parsed_command_line_of_config_file(
            &config_path,
            &CompilerOptions::default(),
            cwd,
            fs.as_ref(),
        ),
        config_path,
    ))
}
