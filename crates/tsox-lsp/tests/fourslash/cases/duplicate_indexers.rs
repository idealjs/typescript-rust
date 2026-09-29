use tsox_lsp::fourslash::{self, Session};


#[test]
fn duplicate_indexers() {
    let content = r#"interface I {
    [x: number]: string;
    [x: number]: number;
}
var i: I;
var /**/r = i[1];"#;
    let mut s = Session::new_for_test("duplicateIndexers", content);
    fourslash::verify_quick_info_at(&mut s, "", "var r: string", "");
}
