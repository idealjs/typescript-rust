use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_import_type_meaning_at_location() {
    let content = r#"// @Filename: /a.ts
/*1*/export type /*2*/T = 0;
/*3*/export const /*4*/T = 0;
// @Filename: /b.ts
const x: import("./a")./*5*/T = 0;
const x: typeof import("./a")./*6*/T = 0;"#;
    let mut s = Session::new_for_test("findAllRefs_importType_meaningAtLocation", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
