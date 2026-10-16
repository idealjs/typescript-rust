use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_default_import() {
    let content = r#"// @Filename: /a.ts
export default function /*0*/a() {}
// @Filename: /b.ts
import /*1*/a, * as ns from "./a";"#;
    let _s = Session::new_for_test("findAllRefsDefaultImport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
