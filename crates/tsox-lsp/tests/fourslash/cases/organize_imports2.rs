use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports2() {
    let content = r#"import {
    Foo   
 , Bar   
} from "foo"

console.log(Foo, Bar);"#;
    let _s = Session::new_for_test("organizeImports2", content);
    // TODO: f.VerifyOrganizeImports(t,
}
