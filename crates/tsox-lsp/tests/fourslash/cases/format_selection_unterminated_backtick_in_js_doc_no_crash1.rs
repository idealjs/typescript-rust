use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_unterminated_backtick_in_js_doc_no_crash1() {
    let content = r#"// @allowJs: true
// @filename: index.js
export class Manifest {
  /**
   * @template {ExtensionType} ExtType
   * @param {ExtType} extType - `dri
/*begin*/   * @param {string} extName
   */
  setExtension(extType, extName, extData) {
        const data = _.cloneDeep(extData);
    this.#data[`${extType}s`][extName] = data;
    return data;
  }
/*end*/
}

"#;
    let mut s = Session::new_for_test("formatSelectionUnterminatedBacktickInJSDocNoCrash1", content);
    fourslash::format_selection(&mut s, "begin", "end");
    // TODO: f.VerifyCurrentFileContent(t, "export class Manifest {\n"+
}
