use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
#[test]
fn missing_method_after_edit_after_import() {
    let content = r#"namespace foo {
    export namespace bar { namespace baz { export class boo { } } }
}

import f = /*foo*/foo;

/*delete*/var x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "foo", "namespace foo", "")
    fourslash::go_to_marker(&mut s, "delete");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 6)
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "foo", "namespace foo", "")
}
