use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineLinkedEditing"]
#[test]
fn linked_editing_jsx_tag12() {
    let content = r#"// @Filename: /incomplete.tsx
function Test() {
    return <div>
        </*0*/
        <div {...{}}>
        </div>
    </div>
}
// @Filename: /incompleteMismatched.tsx
function Test() {
    return <div>
        <T
        <div {...{}}>
        </div>
    </div>
}
// @Filename: /incompleteMismatched2.tsx
function Test() {
    return <div>
        <T
        <div {...{}}>
        T</div>
    </div>
}
// @Filename: /incompleteMismatched3.tsx
function Test() {
    return <div>
        <div {...{}}>
        </div>
        <T
    </div>
}
// @Filename: /mismatched.tsx
function Test() {
    return <div>
        <T>
        <div {...{}}>
        </div>
    </div>
}
// @Filename: /matched.tsx
function Test() {
    return <div>

        <div {...{}}>
        </div>
    </div>
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyLinkedEditing"); // f.VerifyLinkedEditing(t, map[string][]lsproto.Range{"0": nil})
    fourslash::unsupported("VerifyBaselineLinkedEditing"); // f.VerifyBaselineLinkedEditing(t)
}
