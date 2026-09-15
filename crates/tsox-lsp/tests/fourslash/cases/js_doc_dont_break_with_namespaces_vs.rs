use tsox_lsp::fourslash::Session;


#[test]
fn js_doc_dont_break_with_namespaces_vs() {
    let content = r#"// @allowJs: true
// @Filename: jsDocDontBreakWithNamespaces.js
/**
 * @returns {module:@nodefuel/web~Webserver~wsServer#hello} Websocket server object
 */
function foo() { }
foo(''/*foo*/);

/**
 * @type {module:xxxxx} */
 */
function bar() { }
bar(''/*bar*/);

/** @type {function(module:xxxx, module:xxxx): module:xxxxx} */
function zee() { }
zee(''/*zee*/);"#;
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
