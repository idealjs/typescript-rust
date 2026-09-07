use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_object_literal_with_accessors() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function /*1*/makePoint(x: number) {
    return {
        b: 10,
        get x() { return x; },
        set x(a: number) { this.b = a; }
    };
};
var /*4*/point = makePoint(2);
var /*2*/x = point.x;
point./*3*/x = 30;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "function makePoint(x: number): {\n    b: number;\n    x: number;\n}",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "2", "var x: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(property) x: number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "4",
        "var point: {\n    b: number;\n    x: number;\n}",
        "",
    );
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
