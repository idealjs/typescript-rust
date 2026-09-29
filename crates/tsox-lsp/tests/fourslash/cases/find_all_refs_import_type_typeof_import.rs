use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_import_type_typeof_import() {
    let content = r#"// @Filename: /a.ts
export const x = 0;
// @Filename: /b.ts
/*1*/const x: typeof import("/*2*/./a") = { x: 0 };
/*3*/const y: typeof import("/*4*/./a") = { x: 0 };"#;
    let _s = Session::new_for_test("findAllRefs_importType_typeofImport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
