use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_root_symbols() {
    let content = r#"interface I { /*0*/x: {}; }
interface J { /*1*/x: {}; }
declare const o: (I | J) & { /*2*/x: string };
o./*3*/x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
