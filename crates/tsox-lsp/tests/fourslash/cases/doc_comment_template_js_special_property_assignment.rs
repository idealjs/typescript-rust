use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn doc_comment_template_js_special_property_assignment() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/*0*/module.exports = function(a) {};
const myNamespace  = {};
/*1*/myNamespace.myExport = function(x) {};"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(false)
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyJSDocCompletion(t, "0", 7, `/**
    // TODO: f.VerifyJSDocCompletion(t, "1", 7, `/**
}
