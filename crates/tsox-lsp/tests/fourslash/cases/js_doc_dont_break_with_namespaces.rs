use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_dont_break_with_namespaces() {
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
    let mut s = Session::new_for_test("jsDocDontBreakWithNamespaces", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
