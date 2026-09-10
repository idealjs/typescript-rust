use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // (e.g. Record<string, string>) should show the value type "]
#[test]
fn quick_info_index_signature_mapped_type() {
    // TODO: // Regression test for https://github.com/microsoft/TypeScript/tsc/issues/3018
    // TODO: // Quick info for property access resolved from an index signature on a mapped type
    // TODO: // (e.g. Record<string, string>) should show the value type rather than nothing.
    let content = r#"
// @strict: true
// @filename: main.ts
declare const record: Record<string, string>;
record.fo/*1*/o;
"#;
    let mut s = Session::new_for_test("quickInfoIndexSignatureMappedType", content);
    fourslash::verify_quick_info_at(&mut s, "1", "string", "");
}
