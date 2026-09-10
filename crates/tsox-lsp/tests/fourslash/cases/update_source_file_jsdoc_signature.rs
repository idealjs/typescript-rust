use tsox_lsp::fourslash::{self, Session};


#[test]
fn update_source_file_jsdoc_signature() {
    let content = r#"/**
 * @callback Cb
 * @return {/**/}
 */
let x;"#;
    let mut s = Session::new_for_test("updateSourceFile_jsdocSignature", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "number");
}
