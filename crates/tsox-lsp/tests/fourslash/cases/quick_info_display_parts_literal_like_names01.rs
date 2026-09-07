use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn quick_info_display_parts_literal_like_names01() {
    let content = r#"class C {
    public /*1*/1() { }
    private /*2*/Infinity() { }
    protected /*3*/NaN() { }
    static /*4*/"stringLiteralName"() { }
    method() {
        this[/*5*/1]();
        this[/*6*/"1"]();
        this./*7*/Infinity();
        this[/*8*/"Infinity"]();
        this./*9*/NaN();
        C./*10*/stringLiteralName();
    }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
    // TODO: }
}
