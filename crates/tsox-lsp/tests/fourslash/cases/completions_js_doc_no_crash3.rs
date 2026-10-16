use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_no_crash3() {
    let content = r#"// @strict: true
// @filename: index.ts
class MssqlClient {
  /**
   *
   * @param {Object} - args
   * @param {String} - args.parentTable
   * @returns {Promise<{upStatement/**/, downStatement}>}
   */
  async relationCreate(args) {}
}

export default MssqlClient;"#;
    let mut s = Session::new_for_test("completionsJSDocNoCrash3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
