use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_with_local_variables() {
    let content = r#"function x(){
	const x = Object()
	x.foo = ""
}"#;
    let mut s = Session::new_for_test("navigationBarWithLocalVariables", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
