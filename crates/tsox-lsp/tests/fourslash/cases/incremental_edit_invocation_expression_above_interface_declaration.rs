use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn incremental_edit_invocation_expression_above_interface_declaration() {
    let content = r#"// @lib: es5
declare function alert(message?: any): void;
/*1*/
interface Foo {
    setISO8601(dString): Date;
}"#;
    let mut s = Session::new_for_test("incrementalEditInvocationExpressionAboveInterfaceDeclaration", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Insert"); // f.Insert(t, "alert(")
}
