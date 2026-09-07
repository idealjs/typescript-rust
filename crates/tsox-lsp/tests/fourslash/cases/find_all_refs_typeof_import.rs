use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_typeof_import() {
    let content = r#"// @Filename: /a.ts
/*1*/export const /*2*/x = 0;
declare const a: typeof import("./a");
a./*3*/x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
