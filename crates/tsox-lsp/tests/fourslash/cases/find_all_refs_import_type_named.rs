use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_import_type_named() {
    let content = r#"// @Filename: /a.ts
/*1*/export type /*2*/T = number;
/*3*/export type /*4*/U = string;
// @Filename: /b.ts
const x: import("./a")./*5*/T = 0;
const x: import("./a")./*6*/U = 0;"#;
    let mut s = Session::new_for_test("findAllRefs_importType_named", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
