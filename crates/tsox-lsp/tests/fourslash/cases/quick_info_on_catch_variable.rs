use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_catch_variable() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
function f() {
   try { } catch (/**/e) { }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "(local var) e: any", "");
}
