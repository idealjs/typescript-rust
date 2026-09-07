use tsox_lsp::fourslash::parse::parse_test_data;

#[test]
fn strip_trailing_empty_marker() {
    let d = parse_test_data("var fn = () => () => null/**/", "main.ts");
    assert_eq!(d.files[0].content, "var fn = () => () => null");
    assert_eq!(d.markers.len(), 1);
    assert_eq!(d.markers[0].position, 25);
}

#[test]
fn marker_and_range() {
    let d = parse_test_data("const a = /*m*/1; [|sel|]", "f.ts");
    assert_eq!(d.files[0].content, "const a = 1; sel");
    assert_eq!(d.marker_positions.get("m"), Some(&10));
    assert_eq!(d.ranges.len(), 1);
}
