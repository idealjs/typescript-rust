use tsox_lsp::fourslash::Session;


#[test]
fn linked_editing_jsx_tag5() {
    let content = r#"// @FileName: /unclosedElement.tsx
const jsx = (
    <div/*0*/>
        </*1start*/div/*1*/>
    <//*2start*/div/*2*/>/*3*/
);/*4*/
// @FileName: /mismatchedElement.tsx
const jsx = (
    /*5*/</*6start*/div/*6*/>
        <//*7start*/div/*7*/>
    </*8*//div/*9*/>/*10*/
);
// @Filename: /invalidClosing.tsx
const jsx = (
   <di/*11*/v>
   </*12*/ //*13*/div>
);"#;
    let _s = Session::new_for_test("linkedEditingJsxTag5", content);
    // TODO: linkedCursors1 := []lsproto.Range{
    // TODO: linkedCursors2 := []lsproto.Range{
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
