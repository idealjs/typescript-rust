use tsox_lsp::fourslash::Session;


#[test]
fn linked_editing_jsx_tag1() {
    let content = r#"// @Filename: /basic.tsx
/*a*/const j/*b*/sx = (
    /*c*/</*0*/d/*1*/iv/*2*/>/*3*/
    </*4*///*5*/di/*6*/v/*7*/>/*8*/
);
const jsx2 = (
    </*9start*/d/*9*/iv/*9end*/>
        </*10start*/d/*10*/iv/*10end*/>
            </*11start*/p/*11*/>
            <//*12*/p/*12end*/>        
        <//*13start*/d/*13*/iv/*13end*/>
    <//*14start*/d/*14*/iv/*14end*/>
);/*d*/"#;
    let _s = Session::new_for_test("linkedEditingJsxTag1", content);
    // TODO: linkedCursors1 := []lsproto.Range{
    // TODO: linkedCursors2 := []lsproto.Range{
    // TODO: linkedCursors3 := []lsproto.Range{
    // TODO: linkedCursors4 := []lsproto.Range{
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
