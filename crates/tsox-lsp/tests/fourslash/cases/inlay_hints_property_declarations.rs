use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_property_declarations() {
    let content = r#"// @strict: true
class C {
    a = 1
    b: number = 2
    c;
    d;

    constructor(value: number) {
        this.d = value;
        if (value <= 0) {
            this.d = null;
        }
    }
}"#;
    let _s = Session::new_for_test("inlayHintsPropertyDeclarations", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
