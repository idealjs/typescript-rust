use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_object_literal_with_only_setter() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function /*1*/makePoint(x: number) {
    return {
        b: 10,
        set x(a: number) { this.b = a; }
    };
};
var /*3*/point = makePoint(2);
point./*2*/x = 30;"#;
    let mut s = Session::new_for_test("quickInfoOnObjectLiteralWithOnlySetter", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "1", "function makePoint(x: number): {\n    b: number;\n    x: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) x: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var point: {\n    b: number;\n    x: number;\n}", "");
}
