use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_aliased_generic() {
    let content = r#"namespace M {
    export namespace N {
        export class C<T> { }
        export class D { }
    }
}
import d = M.N;
var /*1*/aa: d.C<number>;
var /*2*/bb: d.D;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var aa: d.C<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var bb: d.D", "")
}
