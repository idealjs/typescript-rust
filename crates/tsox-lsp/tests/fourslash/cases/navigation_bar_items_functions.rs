use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_items_functions() {
    let content = r#"function foo() {
    var x = 10;
    function bar() {
        var y = 10;
        function biz() {
            var z = 10;
        }
        function qux() {
            // A function with an empty body should not be top level
        }
    }
}

function baz() {
    var v = 10;
}"#;
    let mut s = Session::new_for_test("navigationBarItemsFunctions", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
