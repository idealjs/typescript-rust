use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_object_literal_with_only_getter() {
    // TODO: t.Skip("Known failing fourslash test")
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
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
