use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_default_export_anonymous() {
    let content = r#"// @Filename: /a.ts
export /*1*/default 1;
// @Filename: /b.ts
import a from "./a";"#;
    let _s = Session::new_for_test("findAllRefsForDefaultExport_anonymous", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
