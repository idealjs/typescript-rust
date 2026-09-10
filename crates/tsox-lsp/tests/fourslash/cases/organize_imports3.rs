use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports3() {
    let content = r#"import {
    Bar   
    , Foo   
  } from "foo"

console.log(Foo, Bar);"#;
    let mut s = Session::new_for_test("organizeImports3", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
