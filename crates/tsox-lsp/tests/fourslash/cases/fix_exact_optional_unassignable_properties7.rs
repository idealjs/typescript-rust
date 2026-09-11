use tsox_lsp::fourslash::{self, Session};


#[test]
fn fix_exact_optional_unassignable_properties7() {
    let content = r#"// @strictNullChecks: true
// @exactOptionalPropertyTypes: true
// @Filename: fixExactOptionalUnassignableProperties6.ts
class Feh {
    _requestFinished(error?: string) {
        this._finishedPromiseCallback({ error/**/ });
    }
    private _finishedPromiseCallback: (arg: { error?: string }) => void = () => {};
}"#;
    let mut s = Session::new_for_test("fixExactOptionalUnassignableProperties7", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
