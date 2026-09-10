use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn organize_imports_type6() {
    let content = r#"import { type a, A, b } from "foo";
interface Use extends A {}
console.log(a, b);"#;
    let mut s = Session::new_for_test("organizeImportsType6", content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type a, A, b } from \"foo1\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type a, A, b } from \"foo2\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type a, A, b } from \"foo3\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
