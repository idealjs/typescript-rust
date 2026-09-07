use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_object_spread() {
    let content = r#"interface A1 { readonly /*0*/a: string };
interface A2 { /*1*/a?: number };
let a1: A1;
let a2: A2;
let a12 = { ...a1, ...a2 };
a12./*2*/a;
a1./*3*/a;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
