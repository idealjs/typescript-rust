use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_object_literal_with_partial_constraint() {
    let content = r#"interface MyOptions {
    hello?: boolean;
    world?: boolean;
}
declare function bar<T extends MyOptions>(options?: Partial<T>): void;
bar({ hello: true, /*1*/ });

interface Test {
    keyPath?: string;
    autoIncrement?: boolean;
}

function test<T extends Record<string, Test>>(opt: T) { }

test({
    a: {
        keyPath: 'x.y',
        autoIncrement: true
    },
    b: {
        /*2*/
    }
});
type Colors = {
    rgb: { r: number, g: number, b: number };
    hsl: { h: number, s: number, l: number }
};

function createColor<T extends keyof Colors>(kind: T, values: Colors[T]) { }

createColor('rgb', {
  /*3*/
});

declare function f<T extends 'a' | 'b', U extends { a?: string }, V extends { b?: string }>(x: T, y: { a: U, b: V }[T]): void;

f('a', {
  /*4*/
});

declare function f2<T extends { x?: string }>(x: T): void;
f2({
  /*5*/
});

type X = { a: { a }, b: { b } }

function f4<T extends 'a' | 'b'>(p: { kind: T } & X[T]) { }

f4({
    kind: "a",
    /*6*/
})"#;
    let mut s = Session::new_for_test("completionsObjectLiteralWithPartialConstraint", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["b", "g", "r"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("6"), &["a"]);
}
