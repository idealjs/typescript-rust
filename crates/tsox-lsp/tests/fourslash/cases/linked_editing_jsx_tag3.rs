use tsox_lsp::fourslash::{self, Session};


#[test]
fn linked_editing_jsx_tag3() {
    let content = r#"// @Filename: /selfClosing.tsx
/*0*/const jsx = /*1*/(
   <div> /*2*/
      <p>/*3*/
         No lin/*4*/ked cursors here!
         /*5*/</*6*/img/*7*/ /*8*///*9*/>
     /*10*/ </p>/*11*/
   /*12*/</div>
/*13*/)/*14*/;/*15*/"#;
    let mut s = Session::new_for_test("linkedEditingJsxTag3", content);
    // TODO: f.VerifyLinkedEditing(t, map[string][]lsproto.Range{
}
