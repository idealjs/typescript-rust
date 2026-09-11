use tsox_lsp::fourslash::{self, Session};


#[test]
fn private_property_of_undefined_this1() {
    let content = r#"
// @strict: true
this.#a = {};
export {};
"#;
    let mut s = Session::new_for_test("privatePropertyOfUndefinedThis1", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}

#[test]
fn private_property_of_undefined_this2() {
    let content = r#"
// @strict: true
export class C {
    wat(this: any): boolean {
        return this.#prop = {};
    }
}
"#;
    let mut s = Session::new_for_test("privatePropertyOfUndefinedThis2", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
