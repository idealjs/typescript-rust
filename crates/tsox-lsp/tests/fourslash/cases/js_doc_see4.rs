use tsox_lsp::fourslash::Session;


#[test]
fn js_doc_see4() {
    let content = r#"class [|/*def1*/A|] {
    foo () { }
}
declare const [|/*def2*/a|]: A;
/**
 * @see {/*use1*/[|A|]#foo}
 */
const t1 = 1
/**
 * @see {/*use2*/[|a|].foo()}
 */
const t2 = 1
/**
 * @see {@link /*use3*/[|a|].foo()}
 */
const t3 = 1"#;
    let _s = Session::new_for_test("jsDocSee4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "use1", "use2", "use3")
}
