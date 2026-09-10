use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn js_doc_see_rename1() {
    let content = r#"[|interface [|{| "contextRangeIndex": 0 |}A|] {}|]
/**
 * @see {[|A|]}
 */
declare const a: [|A|]"#;
    let mut s = Session::new_for_test("jsDocSee_rename1", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
}
