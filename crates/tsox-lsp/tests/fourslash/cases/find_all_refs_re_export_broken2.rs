use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_export_broken2() {
    let content = r#"// @Filename: /a.ts
/*1*/export { /*2*/x } from "nonsense";"#;
    let mut s = Session::new_for_test("findAllRefsReExport_broken2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
