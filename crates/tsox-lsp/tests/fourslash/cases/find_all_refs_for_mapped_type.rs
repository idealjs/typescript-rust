use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_mapped_type() {
    let content = r#"interface T { /*1*/a: number };
type U = { [K in keyof T]: string };
type V = { [K in keyof U]: boolean };
const u: U = { a: "" }
const v: V = { a: true }"#;
    let mut s = Session::new_for_test("findAllRefsForMappedType", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
