use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToBOF"]
#[test]
fn type_above_number_literal_expression_statement() {
    let content = r#"
// foo
1;"#;
    let mut s = Session::new_for_test("typeAboveNumberLiteralExpressionStatement", content);
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::insert(&mut s, "var x;\n");
}
