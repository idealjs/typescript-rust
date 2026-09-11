use tsox_lsp::fourslash::{self, Session};


#[test]
fn linked_editing_jsx_tag7() {
    let content = r#"// @FileName: /fragment.tsx
/*a*/const j/*b*/sx =/*c*/ (
    /*5*/</*0*/>/*1*/
        <img />
    /*6*/</*2*///*3*/>/*4*/
)/*d*/;
const jsx2 = (
    /* this is comment *//*13*/</*10*//* /*11*/more comment *//*12*/>/*8*/Hello/*9*/
    <//*14*/ /*18*///*17*/* even/*15*/ more comment *//*16*/>
);
const jsx3 = (
    <>/*7*/
    </>
);/*e*/"#;
    let mut s = Session::new_for_test("linkedEditingJsxTag7", content);
    // TODO: startRange := f.MarkerByName(t, "0").LSPosition
    // TODO: endRange := f.MarkerByName(t, "3").LSPosition
    // TODO: linkedCursors1 := []lsproto.Range{
    // TODO: startRange2 := f.MarkerByName(t, "10").LSPosition
    // TODO: endRange2 := f.MarkerByName(t, "14").LSPosition
    // TODO: linkedCursors2 := []lsproto.Range{
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
