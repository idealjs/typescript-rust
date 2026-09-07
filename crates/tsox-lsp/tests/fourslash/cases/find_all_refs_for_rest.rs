use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_rest() {
    let content = r#"interface Gen {
    x: number
    /*1*/parent: Gen;
    millenial: string;
}
let t: Gen;
var { x, ...rest } = t;
rest./*2*/parent;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
