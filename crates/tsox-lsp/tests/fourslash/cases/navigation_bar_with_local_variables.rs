use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_with_local_variables() {
    let content = r#"function x(){
	const x = Object()
	x.foo = ""
}"#;
    let mut s = Session::new_for_test("navigationBarWithLocalVariables", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
