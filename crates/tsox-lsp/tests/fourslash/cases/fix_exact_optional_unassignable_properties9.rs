use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn fix_exact_optional_unassignable_properties9() {
    let content = r#"// @strictNullChecks: true
// @exactOptionalPropertyTypes: true
interface IAny {
    a?: any
}
interface J {
    a?: number | undefined
}
declare var iany: IAny
declare var j: J
iany/**/ = j"#;
    let mut s = Session::new_for_test("fixExactOptionalUnassignableProperties9", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
