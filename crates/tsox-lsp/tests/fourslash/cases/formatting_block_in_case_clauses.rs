use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_block_in_case_clauses() {
    let content = r#"switch (1) {
    case 1:
        {
            /*1*/
        break;
}"#;
    let mut s = Session::new_for_test("formattingBlockInCaseClauses", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "}");
    // TODO: f.VerifyCurrentLineContent(t, `
}
