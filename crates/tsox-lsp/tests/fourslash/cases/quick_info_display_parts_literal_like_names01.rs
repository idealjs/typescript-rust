use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("quickInfoDisplayPartsLiteralLikeNames01", content);
    // TODO: f.VerifyBaselineHover(t)
    // TODO: }
}
