use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn contextually_typed_function_expression_generic1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Comparable<T> {
   compareTo(other: T): T;
}
interface Comparer {
   <T extends Comparable<T>>(x: T, y: T): T;
}
var max2: Comparer = (x/*1*/x, y/*2*/y) => { return x/*3*/x.compareTo(y/*4*/y) };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) xx: T extends Comparable<T>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) yy: T extends Comparable<T>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) xx: T extends Comparable<T>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) yy: T extends Comparable<T>", "")
}
