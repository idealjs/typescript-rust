use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn generic_combinators3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Collection<T, U> {
}

interface Combinators {
    map<T, U, V>(c: Collection<T,U>, f: (x: T, y: U) => V): Collection<T, V>;
    map<T, U>(c: Collection<T,U>, f: (x: T, y: U) => any): Collection<any, any>;
}

var c2: Collection<number, string>;

var _: Combinators;

var /*9*/r1a  = _.ma/*1c*/p(c2, (/*1a*/x,/*1b*/y) => { return x + "" });  // check quick info of map here"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1a", "(parameter) x: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1b", "(parameter) y: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1c", "(method) Combinators.map<number, string, string>(c: Collection<number,
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "9", "var r1a: Collection<number, string>", "")
}
