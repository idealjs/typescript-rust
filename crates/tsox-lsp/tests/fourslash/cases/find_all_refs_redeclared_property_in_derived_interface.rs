use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_redeclared_property_in_derived_interface() {
    let content = r#"// @noLib: true
interface A {
    readonly /*0*/x: number | string;
}
interface B extends A {
    readonly /*1*/x: number;
}
const a: A = { /*2*/x: 0 };
const b: B = { /*3*/x: 0 };"#;
    let mut s = Session::new_for_test("findAllRefsRedeclaredPropertyInDerivedInterface", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
