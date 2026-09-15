use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports3() {
    let content = r#"import {
    Bar   
    , Foo   
  } from "foo"

console.log(Foo, Bar);"#;
    let _s = Session::new_for_test("organizeImports3", content);
    // TODO: f.VerifyOrganizeImports(t,
}
