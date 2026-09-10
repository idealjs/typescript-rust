use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion2() {
    let content = r#"// overloads
declare function foo(a: string): number;
/** @deprecated */
declare function foo(): undefined;
declare function foo (a?: string): number | undefined;
[|foo|]();
foo('');
foo;
/** @deprecated */
declare function bar(): number;
[|bar|]();
[|bar|];
/** @deprecated */
declare function baz(): number;
/** @deprecated */
declare function baz(): number | undefined;
[|baz|]();
[|baz|];
interface Foo {
    /** @deprecated */
    (): void
    (a: number): void
}
declare const f: Foo;
[|f|]();
f(1);
interface T {
    createElement(): void
    /** @deprecated */
    createElement(tag: 'xmp'): void;
}
declare const t: T;
t.createElement();
t.[|createElement|]('xmp');
declare class C {
    /** @deprecated */
    constructor ();
    constructor(v: string)
}
C;
const c = new [|C|]();
interface Ca {
    /** @deprecated */
    (): void
    new (): void
}
interface Cb {
    (): void
    /** @deprecated */
    new (): string
}
declare const ca: Ca;
declare const cb: Cb;
ca;
cb;
[|ca|]();
cb();
new ca();
new [|cb|]();"#;
    let mut s = Session::new_for_test("jsdocDeprecated_suggestion2", content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
