use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyLinkedEditing"]
#[test]
fn linked_editing_jsx_tag8() {
    let content = r#"// @FileName: /mismatchedNames.tsx
const A = thing;
const B = thing;
const jsx = (
    </*8*/A>
    </B>
);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyLinkedEditing"); // f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
