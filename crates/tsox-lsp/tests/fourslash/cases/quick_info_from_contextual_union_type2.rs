use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_from_contextual_union_type2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
function test1(arg: { prop: "foo" }) {}
test1({ /*1*/prop: "bar" });

function test2(arg: { prop: "foo" } | undefined) {}
test2({ /*2*/prop: "bar" });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) prop: \"foo\"", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) prop: \"foo\"", "")
}
