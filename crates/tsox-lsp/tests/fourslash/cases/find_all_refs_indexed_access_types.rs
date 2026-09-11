use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_indexed_access_types() {
    let content = r#"interface I {
    /*1*/0: number;
    /*2*/s: string;
}
interface J {
    a: I[/*3*/0],
    b: I["/*4*/s"],
}"#;
    let mut s = Session::new_for_test("findAllRefsIndexedAccessTypes", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
