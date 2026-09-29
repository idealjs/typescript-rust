use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("linkedEditingJsxTag12", content);
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{"0": nil})
    // TODO: f.VerifyBaselineLinkedEditing(t)
}
