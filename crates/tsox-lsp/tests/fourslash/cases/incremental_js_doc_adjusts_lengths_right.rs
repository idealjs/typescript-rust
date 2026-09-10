use tsox_lsp::fourslash::{self, Session};


#[test]
fn incremental_js_doc_adjusts_lengths_right() {
    let content = r#"// @noLib: true

/**
 * Pad ` + "`" + `str` + "`" + ` to ` + "`" + `width` + "`" + `.
 *
 * @param {String} str
 * @param {Number} wid/*1*/"#;
    let mut s = Session::new_for_test("incrementalJsDocAdjustsLengthsRight", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "th\n@");
}
