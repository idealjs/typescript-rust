use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "1", "var aa: d.C<number>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var bb: d.D", "");
}
