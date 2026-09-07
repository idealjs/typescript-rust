use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_mapped_type_non_homomorphic() {
    let content = r#"// @strict: true
function f(x: { [K in "m"]: number; }) {
    x./*1*/m;
    x./*2*/m
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
