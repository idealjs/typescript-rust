use tsox_lsp::fourslash::Session;


#[test]
fn linked_editing_jsx_tag11() {
    let content = r#"// @Filename: /customElements.tsx
const jsx = <fbt:enum knownProp="accepted"
    unknownProp="rejected">
</fbt:enum>;

const customElement = <custom-element></custom-element>;

const standardElement = 
   <Link href="/hello" passHref>
       <Button component="a">
           Next
       </Button>
   </Link>;"#;
    let _s = Session::new_for_test("linkedEditingJsxTag11", content);
    // TODO: f.VerifyBaselineLinkedEditing(t)
}
