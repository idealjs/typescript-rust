use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports8() {
    let content = r#"import { foo as foo } from "foo";
foo;"#;
    let mut s = Session::new_for_test("organizeImports8", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
