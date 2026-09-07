use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_unique_symbol1() {
    let content = r#"declare const Symbol: () => symbol;
namespace M {
    export const sym = Symbol();
}
namespace N {
    const sym = Symbol();
    export interface I {
        [sym]: number;
        [M.sym]: number;
    }
}

declare const i: N.I;
i[|./**/|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
