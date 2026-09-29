use tsox_lsp::fourslash::Session;


#[test]
fn js_doc_see_rename1() {
    let content = r#"[|interface [|{| "contextRangeIndex": 0 |}A|] {}|]
/**
 * @see {[|A|]}
 */
declare const a: [|A|]"#;
    let _s = Session::new_for_test("jsDocSee_rename1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
}
