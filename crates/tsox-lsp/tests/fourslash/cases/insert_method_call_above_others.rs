use tsox_lsp::fourslash::{self, Session};


#[test]
fn insert_method_call_above_others() {
    let content = r#"/**/ 
paired.reduce();
paired.map(() => undefined);"#;
    let mut s = Session::new_for_test("insertMethodCallAboveOthers", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "paired.reduce();");
}
