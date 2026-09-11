use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal4() {
    let content = r#"// @strictNullChecks: true
interface Thing {
    hello: number;
    world: string;
}

declare function funcA(x : Thing): void;
declare function funcB(x?: Thing): void;
declare function funcC(x : Thing | null): void;
declare function funcD(x : Thing | undefined): void;
declare function funcE(x : Thing | null | undefined): void;
declare function funcF(x?: Thing | null | undefined): void;

funcA({ /*A*/ });
funcB({ /*B*/ });
funcC({ /*C*/ });
funcD({ /*D*/ });
funcE({ /*E*/ });
funcF({ /*F*/ });"#;
    let mut s = Session::new_for_test("completionListInObjectLiteral4", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
