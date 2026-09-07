use super::*;

#[test]
fn build_info_roundtrip() {
    let info = BuildInfo::new(
        &[
            ("/src/foo.ts".to_string(), "const x = 1;".to_string()),
            ("/src/bar.ts".to_string(), "const y = 2;".to_string()),
        ],
        "/src/tsconfig.json",
        "abc123",
        &[],
    );
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: BuildInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.fileNames.len(), 2);
    assert_eq!(deserialized.root, "/src/tsconfig.json");
}

#[test]
fn up_to_date_check() {
    let files = vec![("/src/foo.ts".to_string(), "const x = 1;".to_string())];
    let info = BuildInfo::new(&files, "/src", "hash123", &[]);

    assert!(info.is_up_to_date(&files, "hash123"));

    let changed = vec![("/src/foo.ts".to_string(), "const x = 2;".to_string())];
    assert!(!info.is_up_to_date(&changed, "hash123"));

    assert!(!info.is_up_to_date(&files, "different"));
}

#[test]
fn build_info_file_path() {
    let path = BuildInfo::get_ts_build_info_file_path("/src/tsconfig.json", "/src/dist", "");
    assert_eq!(path, "/src/dist/tsconfig.tsbuildinfo");

    let path = BuildInfo::get_ts_build_info_file_path(
        "/src/tsconfig.json",
        "/src/dist",
        "custom.tsbuildinfo",
    );
    assert_eq!(path, "custom.tsbuildinfo");
}
