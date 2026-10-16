use tsox_lsp::fourslash::Session;


#[test]
fn linked_editing_jsx_tag6() {
    let content = r#"// @Filename: /namespace.tsx
const jsx = (
    </*start*/someNamespa/*3*/ce./*2*/Thing/*startend*/>
    <//*end*/someNamespace/*1*/.Thing/*endend*/>
);
 const jsx1 = </*4*/foo/*5*/  /*6*/./*7*/ /*8*/ba/*9*/r><//*10*/foo.bar>;
 const jsx2 = <foo./*11*/bar><//*12*/ /*13*/f/*14*/oo /*15*/./*16*/b/*17*/ar/*18*/>;
 const jsx3 = </*19*/foo/*20*/ //*21*// /*22*/some comment
     /*23*/./*24*/bar>
     </f/*25*/oo.bar>;
 let jsx4 =
     </*26*/foo  /*27*/ .// hi/*28*/
     /*29*/bar/*26end*/>
     <//*30*/foo  /*31*/ .// hi/*32*/
     /*33*/bar/*30end*/>"#;
    let _s = Session::new_for_test("linkedEditingJsxTag6", content);
    // TODO: linkedCursors1 := []lsproto.Range{
    // TODO: linkedCursors2 := []lsproto.Range{
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
