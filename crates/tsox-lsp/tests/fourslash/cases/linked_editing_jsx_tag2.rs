use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Test file content (for readability):"]
#[test]
fn linked_editing_jsx_tag2() {
    let content = r#"// @Filename: /attrs.tsx
const jsx = (
   </*0*/div/*1*/ /*2*/styl/*3*/e={{ color: 'red' }}/*4*/>/*5*/
      <p>
         <img />
      </p>
   <//*6start*/di/*6*/v/*6end*/>
);
// @Filename: /attrsError.tsx
const jsx = (
   </*10*/div/*11*/ /*12*/styl/*13*/e={{ color: 'red' }/*14*/>/*15*/
         </*16*/p />
   <//*17*/div>
);"#;
    let mut s = Session::new(content);
    // TODO: // Test file content (for readability):
    // TODO: // const jsx = (
    // TODO: linkedCursors := []lsproto.Range{
    fourslash::unsupported("VerifyLinkedEditing"); // f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
    // TODO: }
}
