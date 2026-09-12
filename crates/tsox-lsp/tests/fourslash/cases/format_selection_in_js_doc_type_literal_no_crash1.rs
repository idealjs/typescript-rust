use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_in_js_doc_type_literal_no_crash1() {
    let content = r#"// @allowJs: true
// @filename: index.js
/**
 *
 *
 * @typedef {Object} Fixture
 * @property {typeof build} build
/*begin*/ * @property {(url: string) => string} resolveUrl
 * @property {() => Promise<void>} clean
/*end*/ * @property {(streaming?: boolean) => Promise<App>} loadTestAdapterApp
 */

"#;
    let mut s = Session::new_for_test("formatSelectionInJSDocTypeLiteralNoCrash1", content);
    fourslash::format_selection(&mut s, "begin", "end");
    fourslash::verify_current_file_content(&mut s, concat!("/**\n", " *\n", " *\n", " * @typedef {Object} Fixture\n", " * @property {typeof build} build\n", " * @property {(url: string) => string} resolveUrl\n", " * @property {() => Promise<void>} clean\n", " * @property {(streaming?: boolean) => Promise<App>} loadTestAdapterApp\n", " */\n", "\n"));
}
