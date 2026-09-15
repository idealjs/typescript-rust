use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_mapped_type_non_homomorphic() {
    let content = r#"// @strict: true
function f(x: { [K in "m"]: number; }) {
    x./*1*/m;
    x./*2*/m
}"#;
    let _s = Session::new_for_test("findAllRefsMappedType_nonHomomorphic", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
