use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: linkedCursors1 := []lsproto.Range{"]
#[test]
fn linked_editing_jsx_tag9() {
    let content = r#"// @Filename: /whitespace.tsx
const whitespaceOpening = (
   </*0*/ /*1*/div/*2*/ /*3*/> /*4*/
   <//*5*/di/*6*/v/*5end*/>
);
const whitespaceClosing = (
   </*7*/di/*8*/v/*8end*/>
   <//*9*/ /*10*/div/*11*/ /*12*/> /*13*/
);
const triviaOpening = (
    /* this is/*14*/ comment *//*15*/</*16*//* /*17*/more/*18*/ comment *//*19*/ /*20start*/di/*20*/v/*20end*/ /* comments */>/*21*/Hello/*22*/
    <//*23*/ /*24*///*25*/* even/*26*/ more comment *//*27*/ /*28start*/d/*28*/iv/*28end*/ /* b/*29*/ye */>
);"#;
    let mut s = Session::new_for_test("linkedEditingJsxTag9", content);
    // TODO: linkedCursors1 := []lsproto.Range{
    // TODO: linkedCursors2 := []lsproto.Range{
    // TODO: linkedCursors3 := []lsproto.Range{
    fourslash::unsupported("VerifyLinkedEditing"); // f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
