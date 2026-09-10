use tsox_lsp::fourslash::{self, Session};


#[ignore = "needs live LSP session"]
#[test]
fn formatting_of_chained_lambda() {
    let content = r#"var fn = (x: string) => ()=> alert(x)/**/"#;
    let mut s = Session::new_for_test("formattingOfChainedLambda", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ";");
    fourslash::verify_current_line_content(&mut s, r#"var fn = (x: string) => () => alert(x);"#);
}
