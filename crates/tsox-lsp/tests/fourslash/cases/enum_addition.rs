use tsox_lsp::fourslash::{self, Session};


#[test]
fn enum_addition() {
    let content = r#"namespace m { export enum Color { Red } }
var /**/t = m.Color.Red + 1;"#;
    let mut s = Session::new_for_test("enumAddition", content);
    fourslash::verify_quick_info_at(&mut s, "", "var t: number", "");
}
