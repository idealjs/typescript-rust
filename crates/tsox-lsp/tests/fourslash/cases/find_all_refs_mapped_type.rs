use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_mapped_type() {
    let content = r#"interface T { /*1*/a: number; }
type U = { readonly [K in keyof T]?: string };
declare const t: T;
t./*2*/a;
declare const u: U;
u./*3*/a;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
