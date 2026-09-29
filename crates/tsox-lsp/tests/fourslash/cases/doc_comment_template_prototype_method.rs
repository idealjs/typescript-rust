use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_prototype_method() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
/** @class */
function C() { }
/*above*/
C.prototype.method = /*next*/ function (p) {}"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: for _, marker := range f.MarkerNames() {
}
