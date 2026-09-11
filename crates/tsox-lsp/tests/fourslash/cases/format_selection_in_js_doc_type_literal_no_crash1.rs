use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_in_js_doc_type_literal_no_crash1() {
    let content = r#"// @allowJs: true\n"#;
    // TODO: "// @filename: index.js\n" +
    // TODO: "/**\n" +
    // TODO: " *\n" +
    // TODO: " *\n" +
    // TODO: " * @typedef {Object} Fixture\n" +
    // TODO: " * @property {typeof build} build\n" +
    // TODO: "/*begin*/ * @property {(url: string) => string} resolveUrl\n" +
    // TODO: " * @property {() => Promise<void>} clean\n" +
    // TODO: "/*end*/ * @property {(streaming?: boolean) => Promise<App>} loadTestAdapterApp\n" +
    // TODO: " */\n" +
    // TODO: "\n"
    let mut s = Session::new_for_test("formatSelectionInJSDocTypeLiteralNoCrash1", content);
    // TODO: f.FormatSelection(t, "begin", "end")
    fourslash::verify_current_file_content(&mut s, concat!("/**\n", " *\n", " *\n", " * @typedef {Object} Fixture\n", " * @property {typeof build} build\n", " * @property {(url: string) => string} resolveUrl\n", " * @property {() => Promise<void>} clean\n", " * @property {(streaming?: boolean) => Promise<App>} loadTestAdapterApp\n", " */\n", "\n"));
}
