use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_global_module_augmentation() {
    let content = r#"// @Filename: /a.ts
export {};
declare global {
    /*1*/function /*2*/f(): void;
}
// @Filename: /b.ts
/*3*/f();"#;
    let mut s = Session::new_for_test("findAllRefsGlobalModuleAugmentation", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
