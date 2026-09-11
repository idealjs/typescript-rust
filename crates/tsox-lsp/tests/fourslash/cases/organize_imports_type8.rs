use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_type8() {
    let content = r#"import { type A, type a, b, B } from "foo";
console.log(a, b, A, B);"#;
    let mut s = Session::new_for_test("organizeImportsType8", content);
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo1\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo2\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo3\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo4\";")
    // TODO: f.VerifyOrganizeImports(t,
    // TODO: f.ReplaceLine(t, 0, "import { type A, type a, b, B } from \"foo5\";")
    // TODO: f.VerifyOrganizeImports(t,
}
