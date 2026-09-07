use super::*;

#[test]
fn test_parse_anonymous_marker() {
    let t = FourslashTest::new("function foo() {}");
    let f = &t.files[0];
    assert_eq!(f.content, "function foo() {}");
    assert!(f.markers.is_empty());
    assert_eq!(f.filename, DEFAULT_FILENAME);
}

#[test]
fn test_parse_marker_offsets() {
    let t = FourslashTest::new("function /**/foo(): number { return 1; }");
    let f = &t.files[0];
    assert_eq!(f.content, "function foo(): number { return 1; }");
    assert_eq!(f.markers.len(), 1);
    let pos = t.get_marker("");
    assert_eq!(&f.content[pos..pos + 3], "foo");
}

#[test]
fn test_named_markers() {
    let t = FourslashTest::new("let /*a*/x = 1; let /*b*/y = 2;");
    let f = &t.files[0];
    assert_eq!(f.content, "let x = 1; let y = 2;");
    let a = t.get_marker("a");
    let b = t.get_marker("b");
    assert_eq!(&f.content[a..a + 1], "x");
    assert_eq!(&f.content[b..b + 1], "y");
}

#[test]
fn test_range_markers() {
    let t = FourslashTest::new("const s = [|sel|world|];");
    let f = &t.files[0];
    assert_eq!(f.content, "const s = world;");
    assert_eq!(f.ranges.len(), 1);
    let r = &f.ranges[0];
    assert_eq!(r.name, "sel");
    assert_eq!(&f.content[r.start..r.end], "world");
}

#[test]
fn test_filename_directive_splits_files() {
    let src = "// @filename: a.ts\nexport const shared = 1;\n// @filename: b.ts\nconst local = 2;";
    let t = FourslashTest::new(src);
    assert_eq!(t.files.len(), 2);
    let a = t.get_file("a.ts");
    let b = t.get_file("b.ts");
    assert!(a.content.contains("shared"));
    assert!(b.content.contains("local"));
}

#[test]
fn test_hover_function() {
    let t = FourslashTest::new("function /**/foo(): number { return 1; }");
    let pos = t.get_marker("");
    let file = &t.files[0];
    let hover = t.hover_at(file, pos);
    assert!(hover.contains("foo"), "hover was: {hover:?}");
    assert!(hover.contains("number"), "hover was: {hover:?}");
}

#[test]
fn test_hover_variable() {
    let t = FourslashTest::new("const /**/x = 42;");
    let pos = t.get_marker("");
    let file = &t.files[0];
    let hover = t.hover_at(file, pos);
    assert!(hover.contains("x"), "hover was: {hover:?}");
    assert!(hover.contains("42"), "hover was: {hover:?}");
}

#[test]
fn test_completion_basic() {
    let t = FourslashTest::new("const alpha = 1;\nconst beta = 2;\n/**/\n");
    let pos = t.get_marker("");
    let file = &t.files[0];
    let labels = t.completions_at(file, pos);
    assert!(labels.iter().any(|l| l == "alpha"), "labels: {labels:?}");
    assert!(labels.iter().any(|l| l == "beta"), "labels: {labels:?}");
}

#[test]
fn test_definition() {
    let t =
        FourslashTest::new("function greet(): string { return \"hi\"; }\nconst s = gr/**/eet();");
    let pos = t.get_marker("");
    let file = &t.files[0];
    let (fname, offset) = t
        .definition_at(file, pos)
        .expect("definition should resolve");
    assert_eq!(fname, file.filename);

    assert_eq!(offset, 0, "definition offset");
}

#[test]
fn test_multi_file_program() {
    let src = "// @filename: a.ts\nexport const shared = 1;\n// @filename: b.ts\nconst local = 2;";
    let t = FourslashTest::new(src);
    assert_eq!(t.files.len(), 2);

    let program = t.build_program();
    assert_eq!(program.source_files().len(), 2);
    assert!(program.get_source_file("/proj/a.ts").is_some());
    assert!(program.get_source_file("/proj/b.ts").is_some());
}
