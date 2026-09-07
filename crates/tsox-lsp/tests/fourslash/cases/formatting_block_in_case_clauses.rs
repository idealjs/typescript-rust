use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.Insert(t, '}')"]
#[test]
fn formatting_block_in_case_clauses() {
    let content = r#"switch (1) {
    case 1:
        {
            /*1*/
        break;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.Insert(t, "}")
}
