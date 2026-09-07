use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_binding_pattern_in_jsdoc_no_crash1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/** @type {({ /*1*/data: any }?) => { data: string[] }} */
function useQuery({ data }): { data: string[] } {
  return {
    data,
  };
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "", "");
}
