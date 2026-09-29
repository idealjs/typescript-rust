use tsox_lsp::fourslash::Session;


#[test]
fn object_literal_binding_in_parameter() {
    let content = r#"interface I { x1: number; x2: string }
function f(cb: (ev: I) => any) { }
f(({/*1*/}) => 0);
[<I>null].reduce(({/*2*/}, b) => b);
interface Foo {
    m(x: { x1: number, x2: number }): void;
    prop: I;
}
let x: Foo = {
    m({ /*3*/ }) {
    },
    get prop(): I { return undefined; },
    set prop({ /*4*/ }) {
    }
};"#;
    let _s = Session::new_for_test("objectLiteralBindingInParameter", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
