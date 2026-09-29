use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_property_declarations2() {
    let content = r#"// @strict: true
// @target: esnext
class C {
    accessor a = 1
    accessor b: number = 2
    accessor c;
    accessor d;

    constructor(value: number) {
        this.d = value;
        if (value <= 0) {
            this.d = null;
        }
    }
}"#;
    let _s = Session::new_for_test("inlayHintsPropertyDeclarations2", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
