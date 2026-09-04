use super::*;

#[test]
fn test_iofs() {
    use std::path::PathBuf;

    let fs = OsFS;

    let mut tmp = std::env::temp_dir();
    tmp.push(format!("tsox_iovfs_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let file_path: PathBuf = tmp.join("hello.txt");
    let file_str = file_path.to_str().unwrap();
    fs.write_file(file_str, "hello world").unwrap();

    assert!(fs.file_exists(file_str));
    assert!(!fs.directory_exists(file_str));

    let content = fs.read_file(file_str).expect("read_file returned None");
    assert_eq!(content, "hello world");

    assert_eq!(
        fs.use_case_sensitive_file_names(),
        cfg!(not(target_os = "windows"))
    );

    let entries = fs.get_accessible_entries(tmp.to_str().unwrap());
    assert!(
        entries.files.iter().any(|f| f == "hello.txt"),
        "expected hello.txt in entries.files: {entries:?}"
    );

    let real = fs.realpath(file_str);
    assert!(real.ends_with("hello.txt"), "realpath was {real:?}");

    fs.remove(file_str).unwrap();
    assert!(!fs.file_exists(file_str));

    let _ = std::fs::remove_dir_all(&tmp);
}
