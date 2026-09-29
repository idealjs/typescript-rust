use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quickinfo01() {
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
    let mut s = Session::new_for_test("quickinfo01", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_quick_info_at(&mut s, "1", "var x: One | Two", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) commonProperty: string | number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(method) commonFunction(): number", "");
}
