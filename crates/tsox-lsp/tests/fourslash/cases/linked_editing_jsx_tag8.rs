use tsox_lsp::fourslash::Session;


#[test]
fn linked_editing_jsx_tag8() {
    let content = r#"// @FileName: /mismatchedNames.tsx
const A = thing;
const B = thing;
const jsx = (
    </*8*/A>
    </B>
);"#;
    let _s = Session::new_for_test("linkedEditingJsxTag8", content);
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
