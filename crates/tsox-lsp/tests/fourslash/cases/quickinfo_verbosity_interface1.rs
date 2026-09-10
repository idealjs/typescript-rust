use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_interface1() {
    let content = r#"{
    interface Foo {
        a: "a" | "c";
    }
    const f/*f1*/: Foo = { a: "a" };
}
{
    interface Bar {
        b: "b" | "d";
    }
    interface Foo extends Bar {
        a: "a" | "c";
    }
    const f/*f2*/: Foo = { a: "a", b: "b" };
}
{
    type BarParam = "b" | "d";
    interface Bar {
        bar(b: BarParam): string;
    }
    type FooType = "a" | "c";
    interface FooParam {
        param: FooType;
    }
    interface Foo extends Bar {
        a: FooType;
        foo: (a: FooParam) => number;
    }
    const f/*f3*/: Foo = { a: "a", bar: () => "b", foo: () => 1 };
}
{
    interface Bar<B> {
        bar(b: B): string;
    }
    interface FooParam {
        param: "a" | "c";
    }
    interface Foo extends Bar<FooParam> {
        a: "a" | "c";
        foo: (a: FooParam) => number;
    }
    const f/*f4*/: Foo = { a: "a", bar: () => "b", foo: () => 1 };
    const b/*b1*/: Bar<number> = { bar: () => "" };
}
{
    interface Foo<A> {
        a: A;
    }
    type Alias = Foo<string>;
    const a/*a*/: Alias = { a: "a" };
}
{
    interface Foo {
        a: "a";
    }
    interface Foo {
        b: "b";
    }
    const f/*f5*/: Foo = { a: "a", b: "b" };
}"#;
    let mut s = Session::new_for_test("quickinfoVerbosityInterface1", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"f1": {0, 1}, "f2": {0, 1}, "f3": {0, 1, 2, 3
}
