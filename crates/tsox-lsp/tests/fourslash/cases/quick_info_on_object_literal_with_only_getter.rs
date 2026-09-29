use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_object_literal_with_only_getter() {
    let content = r#"function /*1*/makePoint(x: number) {
    return {
        get x() { return x; },
    };
};
var /*4*/point = makePoint(2);
var /*2*/x = point./*3*/x;"#;
    let mut s = Session::new_for_test("quickInfoOnObjectLiteralWithOnlyGetter", content);
    fourslash::verify_quick_info_at(&mut s, "1", "function makePoint(x: number): {\n    readonly x: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var x: number", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var point: {\n    readonly x: number;\n}", "");
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
