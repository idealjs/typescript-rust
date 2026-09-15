use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_anonymous_class_and_function_expressions3() {
    let content = r#"describe('foo', () => {
    test(`a ${1} b ${2}`, () => {})
})

const a = 1;
const b = 2;
describe('foo', () => {
    test(`a ${a} b {b}`, () => {})
})"#;
    let _s = Session::new_for_test("navigationBarAnonymousClassAndFunctionExpressions3", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
