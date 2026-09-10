use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports2() {
    let content = r#"import {
    Foo   
 , Bar   
} from "foo"

console.log(Foo, Bar);"#;
    let mut s = Session::new_for_test("organizeImports2", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
