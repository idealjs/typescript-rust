use tsox_lsp::fourslash::{self, Session};


#[test]
fn linked_editing_jsx_tag4() {
    let content = r#"// @Filename: /typeTag.tsx
const jsx = (
   </*0*/div/*1*/</*2*/T/*3*/>/*4*/>/*5*/
      <p>
         <img />
      </p>
   <//*6*/div/*7*/>
);
// @Filename: /typeTagError.tsx
const jsx = (
   </*10*/div/*11*/</*12*/T/*13*/>/*14*/
      </*15*/p />
   <//*16*/div>
);"#;
    let mut s = Session::new_for_test("linkedEditingJsxTag4", content);
    // TODO: linkedCursors := []lsproto.Range{
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
