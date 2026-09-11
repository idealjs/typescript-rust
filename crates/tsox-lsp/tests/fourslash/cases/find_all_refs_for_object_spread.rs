use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_object_spread() {
    let content = r#"interface A1 { readonly /*0*/a: string };
interface A2 { /*1*/a?: number };
let a1: A1;
let a2: A2;
let a12 = { ...a1, ...a2 };
a12./*2*/a;
a1./*3*/a;"#;
    let mut s = Session::new_for_test("findAllRefsForObjectSpread", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
