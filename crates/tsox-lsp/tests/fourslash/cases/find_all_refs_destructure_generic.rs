use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_destructure_generic() {
    let content = r#"interface I<T> {
    /*0*/x: boolean;
}
declare const i: I<number>;
const { /*1*/x } = i;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1")
}
