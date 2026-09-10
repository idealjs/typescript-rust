use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn destructured_intersection_js_doc() {
    let content = r#"
type X = {
    /** Description of a. */
    a: {}
}

type Y = X & { a: {} }

declare function f({ /*1*/a }: Y): void
"#;
    let mut s = Session::new_for_test("destructuredIntersectionJSDoc", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn destructured_intersection_js_doc_variable() {
    let content = r#"
type X = {
    /** Description of a. */
    a: {}
}

type Y = X & { a: {} }

declare const y: Y;
const { /*1*/a } = y;
"#;
    let mut s = Session::new_for_test("destructuredIntersectionJSDocVariable", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
