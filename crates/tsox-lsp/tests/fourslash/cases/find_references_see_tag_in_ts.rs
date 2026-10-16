use tsox_lsp::fourslash::Session;


#[test]
fn find_references_see_tag_in_ts() {
    let content = r#"function doStuffWithStuff/*1*/(stuff: { quantity: number }) {}

declare const stuff: { quantity: number };
/** @see {doStuffWithStuff} */
if (stuff.quantity) {}"#;
    let _s = Session::new_for_test("findReferencesSeeTagInTs", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
