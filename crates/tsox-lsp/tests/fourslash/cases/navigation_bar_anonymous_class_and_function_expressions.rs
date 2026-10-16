use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_anonymous_class_and_function_expressions() {
    let content = r#"global.cls = class { };
(function() {
    const x = () => {
        // Presence of inner function causes x to be a top-level function.
        function xx() {}
    };
    const y = {
        // This is not a top-level function (contains nothing, but shows up in childItems of its parent.)
        foo: function() {}
    };
    (function nest() {
        function moreNest() {}
    })();
})();
(function() { // Different anonymous functions are not merged
    // These will only show up as childItems.
    function z() {}
    console.log(function() {})
    describe("this", 'function', `is a function`, `with template literal ${"a"}`, () => {});
    [].map(() => {});
})
(function classes() {
    // Classes show up in top-level regardless of whether they have names or inner declarations.
    const cls2 = class { };
    console.log(class cls3 {});
    (class { });
})"#;
    let _s = Session::new_for_test("navigationBarAnonymousClassAndFunctionExpressions", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
