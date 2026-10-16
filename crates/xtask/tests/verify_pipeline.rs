use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn tmp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let d = std::env::temp_dir().join(format!("xtask_test_{tag}_{}_{}", std::process::id(), nanos));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn anchor_detects_first_divergence_and_skips_noise() {
    let dir = tmp_dir("anchor");
    let local = dir.join("local");
    let go = dir.join("go");
    write(&local.join("foo.types"), "//// [foo.ts] ////\n>A : string\n>B : string\n>C : number\n");
    write(&go.join("foo.types"), "//// [foo.ts] ////\n>A : string\n>B : number\n>C : number\n");
    // typeof 侧与裸 any 值行为已知噪声,不构成锚点
    write(&local.join("noise.types"), "==== x ====\n>V : typeof V\n>W : any\n>C : number\n");
    write(&go.join("noise.types"), "==== x ====\n>V : boolean\n>W : boolean\n>C : number\n");

    let out = dir.join("anchor.csv");
    let anchors = xtask::report::anchor::write(&local, &go, &out).unwrap();
    let csv = std::fs::read_to_string(&out).unwrap();
    assert!(csv.contains("foo,3,"), "anchor at line 3 expected: {csv}");
    assert!(csv.contains("L:>B : string || R:>B : number"), "{csv}");
    assert!(!csv.contains("noise"), "noise-only file must not anchor: {csv}");
    assert_eq!(anchors.len(), 1);
}

#[test]
fn types_index_escapes_newlines_and_quotes() {
    let dir = tmp_dir("types");
    let src = dir.join("types");
    write(&src.join("bar.types"), "line1\nq\"uote\nback\\slash");
    let out = dir.join("rust-types.csv");
    let n = xtask::report::types_index::write(&src, &out).unwrap();
    assert_eq!(n, 1);
    let csv = std::fs::read_to_string(&out).unwrap();
    assert!(csv.starts_with("case,line_count,content\nbar,2,\""), "{csv}");
    assert!(csv.contains("line1\\nq\"\"uote\\nback\\\\slash"), "{csv}");
}

#[test]
fn attribution_matches_legacy_semantics() {
    let dir = tmp_dir("attr");
    let base = dir.join("data");
    write(&base.join("corpus-failures.base.csv"), "key\ncompiler/a.ts\ncompiler/b.ts\ncompiler/c.ts\n");
    write(&base.join("corpus-skips.base.csv"), "key\ncompiler/d.ts\n");

    let fails: BTreeSet<String> = ["compiler/a.ts", "compiler/e.ts"]
        .iter().map(|s| s.to_string()).collect();
    let skips: BTreeSet<String> = ["compiler/c.ts", "compiler/f.ts"].iter().map(|s| s.to_string()).collect();
    let timeout: BTreeSet<String> = BTreeSet::from(["compiler/f.ts".to_string()]);

    let a = xtask::verdict::attribute(Some(&base), &fails, &skips, &timeout);
    assert!(a.base_available);
    // c 是 skip→fail 翻转,不计 new FAIL;e 是真新增
    assert_eq!(a.new_fails, vec!["compiler/e.ts".to_string()]);
    // c 转 SKIP 计 real;f 是超时噪声桶
    assert_eq!(a.new_skips.real, vec!["compiler/c.ts".to_string()]);
    assert_eq!(a.new_skips.timeout, vec!["compiler/f.ts".to_string()]);
    // b 不再 FAIL 且未转 SKIP → fixed;b 转 SKIP 与否决定 fixed 口径
    assert_eq!(a.fixed, 1);

    let a2 = xtask::verdict::attribute(None, &fails, &skips, &timeout);
    assert!(!a2.base_available);
}
