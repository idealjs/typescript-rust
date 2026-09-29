use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_binding_pattern_in_jsdoc_no_crash1() {
    let content = r#"/** @type {({ /*1*/data: any }?) => { data: string[] }} */
function useQuery({ data }): { data: string[] } {
  return {
    data,
  };
}"#;
    let mut s = Session::new_for_test("quickInfoBindingPatternInJsdocNoCrash1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "", "");
}
