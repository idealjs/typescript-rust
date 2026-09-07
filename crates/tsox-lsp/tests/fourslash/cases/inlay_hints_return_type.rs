use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_return_type() {
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
}
const a = () => 1
const b = function () { return 1 }
const c = (b) => 1
const d = b => 1"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
