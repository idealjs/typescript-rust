use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_function_parameter_types3() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
