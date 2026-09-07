use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: linkedCursors := []lsproto.Range{"]
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
    let mut s = Session::new(content);
    // TODO: linkedCursors := []lsproto.Range{
    fourslash::unsupported("VerifyLinkedEditing"); // f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
