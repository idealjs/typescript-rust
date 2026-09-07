use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn quickinfo01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
interface One {
    commonProperty: number;
    commonFunction(): number;
}

interface Two {
    commonProperty: string
    commonFunction(): number;
}

var /*1*/x : One | Two;

x./*2*/commonProperty;
x./*3*/commonFunction;"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var x: One | Two", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) commonProperty: string | number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(method) commonFunction(): number", "")
}
