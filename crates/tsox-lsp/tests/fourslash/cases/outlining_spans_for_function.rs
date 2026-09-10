use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outlining_spans_for_function() {
    let content = r#"[|(
    a: number,
    b: number
) => {
    return a + b;
}|];

(a: number, b: number) =>[| {
    return a + b;
}|]

const f1 = function[| (
    a: number
    b: number
) {
    return a + b;
}|]

const f2 = function (a: number, b: number)[| {
    return a + b;
}|]

function f3[| (
    a: number
    b: number
) {
    return a + b;
}|]

function f4(a: number, b: number)[| {
    return a + b;
}|]

class Foo[| {
    constructor[|(
        a: number,
        b: number
    ) {
        this.a = a;
        this.b = b;
    }|]

    m1[|(
        a: number,
        b: number
    ) {
        return a + b;
    }|]

    m1(a: number, b: number)[| {
        return a + b;
    }|]
}|]

declare function foo(props: any): void;
foo[|(
    a =>[| {

    }|]
)|]

foo[|(
    (a) =>[| {

    }|]
)|]

foo[|(
    (a, b, c) =>[| {

    }|]
)|]

foo[|([|
    (a,
     b,
     c) => {

    }|]
)|]"#;
    let mut s = Session::new_for_test("outliningSpansForFunction", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
