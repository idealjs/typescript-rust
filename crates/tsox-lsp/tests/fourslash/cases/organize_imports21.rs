use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports21() {
    let content = r#"// @filename: /a.ts
export interface LocationDefinitions {}
export interface PersonDefinitions {}
// @filename: /b.ts
export {
    /** @deprecated Use LocationDefinitions instead */
    LocationDefinitions as AddressDefinitions,
    LocationDefinitions,
    /** @deprecated Use PersonDefinitions instead */
    PersonDefinitions as NameDefinitions,
    PersonDefinitions,
} from './a';"#;
    let mut s = Session::new_for_test("organizeImports21", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
