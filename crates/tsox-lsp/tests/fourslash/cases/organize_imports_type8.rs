use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.ReplaceLine"]
#[test]
fn organize_imports_type8() {
    let content = r#"import { type A, type a, b, B } from "foo";
console.log(a, b, A, B);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo1\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo2\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo3\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo4\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo5\";")
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
