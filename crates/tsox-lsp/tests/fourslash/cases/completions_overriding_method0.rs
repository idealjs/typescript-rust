use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_overriding_method0() {
    let content = r#"// @newline: LF
// @Filename: a.ts
abstract class ABase {
    abstract foo(param1: string, param2: boolean): Promise<void>;
}

class ASub extends ABase {
    [|f/*a*/|]
}
// @Filename: b.ts
class BBase {
    foo(a: string, b: string): string {
        return a + b;
    }
}

class BSub extends BBase {
    [|f/*b*/|]
}
// @Filename: c.ts
class CBase {
    foo(a: string | number): string {
        return a + "";
    }
}

class CSub extends CBase {
    foo(a: string): string {
        return add;
    }
}

class CSub2 extends CSub {
    [|f/*c*/|]
}
// @Filename: d.ts
abstract class DBase {
    abstract foo(a: string): string;
}

abstract class DSub extends DBase {
    [|f/*d*/|]
}
// @Filename: e.ts
interface EBase {
    foo(a: string): string;
}

class ESub implements EBase {
    [|f/*e*/|]
}
// @Filename: f.ts
interface FBase {
    foo(a: string): string;
}

abstract class FSub implements FBase {
    [|f/*f*/|]
}
// @Filename: g.ts
interface GBase {
    foo(a: string): string;
    foo(a: undefined, b: number): string;
}

class GSub implements GBase {
    [|f/*g*/|]
}
// @Filename: h.ts
class HBase {
    static met(n: number): number {
        return n;
    }
}

class HSub extends HBase {
    /*h1*/
    static /*h2*/
}
// @Filename: i.ts
class IBase {
    met<T>(t: T): T {
        return t;
    }
    metcons<T extends string | number>(t: T): T {
        return t;
    }
}

class ISub extends IBase {
    /*i*/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod0", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "c");
    // TODO: f.VerifyCompletions(t, "c", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "d");
    // TODO: f.VerifyCompletions(t, "d", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "e");
    // TODO: f.VerifyCompletions(t, "e", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "f");
    // TODO: f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "g");
    // TODO: f.VerifyCompletions(t, "g", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "h1");
    // TODO: f.VerifyCompletions(t, "h1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "h2");
    // TODO: f.VerifyCompletions(t, "h2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "i");
    // TODO: f.VerifyCompletions(t, "i", &fourslash.CompletionsExpectedList{
}
