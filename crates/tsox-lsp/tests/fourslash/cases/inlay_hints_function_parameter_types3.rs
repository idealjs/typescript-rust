use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_function_parameter_types3() {
    let content = r#"interface IFoo {
    bar(x?: boolean): void;
}

const a: IFoo = {
    bar: function (x?): void {
        throw new Error("Function not implemented.");
    }
}
class Foo {
    #value = 0;
    get foo(): number { return this.#value; }
    set foo(value) { this.#value = value; }
}"#;
    let _s = Session::new_for_test("inlayHintsFunctionParameterTypes3", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
