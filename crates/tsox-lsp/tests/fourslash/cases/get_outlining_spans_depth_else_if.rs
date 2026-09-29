use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_depth_else_if() {
    let content = r#"if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else if (1)[| {
    1;
}|] else[| {
    1;
}|]"#;
    let _s = Session::new_for_test("getOutliningSpansDepthElseIf", content);
    // TODO: f.VerifyOutliningSpans(t)
}
