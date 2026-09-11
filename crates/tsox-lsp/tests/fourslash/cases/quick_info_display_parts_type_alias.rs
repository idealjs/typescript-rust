use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_type_alias() {
    let content = r#"class /*1*/c {
}
type /*2*/t1 = /*3*/c;
var /*4*/cInstance: /*5*/t1 = new /*6*/c();"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsTypeAlias", content);
    // TODO: f.VerifyBaselineHover(t)
}
