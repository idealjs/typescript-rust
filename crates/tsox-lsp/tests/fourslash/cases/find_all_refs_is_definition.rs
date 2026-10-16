use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_is_definition() {
    let content = r#"declare function foo(a: number): number;
declare function foo(a: string): string;
declare function foo/*1*/(a: string | number): string | number;

function foon(a: number): number;
function foon(a: string): string;
function foon/*2*/(a: string | number): string | number {
    return a
}

foo; foon;

export const bar/*3*/ = 123;
console.log({ bar });

interface IFoo {
    foo/*4*/(): void;
}
class Foo implements IFoo {
    constructor(n: number)
    constructor()
    /*5*/constructor(n: number?) { }
    foo/*6*/(): void { }
    static init() { return new this() }
}"#;
    let _s = Session::new_for_test("findAllRefsIsDefinition", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6")
}
