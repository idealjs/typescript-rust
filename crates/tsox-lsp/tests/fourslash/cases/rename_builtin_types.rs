use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // All of these should fail because they're library/builtin "]
#[test]
fn rename_builtin_types() {
    let content = r#"
const arr: /*1*/Array<number> = [];
const map1: /*2*/Map<string, number> = new Map();
const prom: /*3*/Promise<void> = Promise.resolve();
const str: /*4*/string = "hello";
"#;
    let mut s = Session::new(content);
    // TODO: // All of these should fail because they're library/builtin types
    // TODO: for _, marker := range []string{"1", "2", "3", "4"} {
}
