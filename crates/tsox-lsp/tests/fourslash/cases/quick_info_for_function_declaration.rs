use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_function_declaration() {
    let content = r#"interface A<T> { }

function ma/*makeA*/keA<T>(t: T): A<T> { return null; }

function /*f*/f<T>(t: T) {
    return makeA(t);
}

var x = f(0);
var y = makeA(0);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "makeA", "function makeA<T>(t: T): A<T>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "f", "function f<T>(t: T): A<T>", "")
}
