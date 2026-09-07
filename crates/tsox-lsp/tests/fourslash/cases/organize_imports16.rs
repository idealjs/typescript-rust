use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn organize_imports16() {
    let content = r#"import { a, A, b } from "foo";
interface Use extends A {}
console.log(a, b);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { a, A, b } from \"foo1\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { a, A, b } from \"foo2\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { a, A, b } from \"foo3\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
