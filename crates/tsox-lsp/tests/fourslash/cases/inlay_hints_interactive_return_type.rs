use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_interactive_return_type() {
    let content = r#"function foo1 () {
    return 1
}
function foo2 (): number {
    return 1
}
class C {
    foo() {
        return 1
    }
    bar() {
        return this
    }
}
const a = () => 1
const b = function () { return 1 }
const c = (b) => 1
const d = b => 1"#;
    let _s = Session::new_for_test("inlayHintsInteractiveReturnType", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
