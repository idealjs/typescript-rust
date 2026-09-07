use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn generic_calls_with_optional_params1() {
    let content = r#"class Collection<T> {
    public add(x: T) { }
}
interface Utils {
    fold<T, S>(c: Collection<T>, folder: (s: S, t: T) => T, init?: S): T;
}
var c = new Collection<string>();
var utils: Utils;
var /*1*/r = utils.fold(c, (s, t) => t, "");
var /*2*/r2 = utils.fold(c, (s, t) => t);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var r: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r2: string", "")
}
