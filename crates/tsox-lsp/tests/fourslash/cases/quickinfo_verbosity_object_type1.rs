use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_object_type1() {
    let content = r#"type Str = string | {};
type FooType = Str | number;
type Sym = symbol | (() => void);
type BarType = Sym | boolean;
type Obj = { foo: FooType, bar: BarType, str: Str };
const obj1/*o1*/: Obj = { foo: 1, bar: true, str: "3"};
const obj2/*o2*/: { foo: FooType, bar: BarType, str: Str } = { foo: 1, bar: true, str: "3"};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"o1": {0, 1, 2, 3}, "o2": {0, 1, 2}})
}
