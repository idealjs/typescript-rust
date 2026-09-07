use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineLinkedEditing"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineLinkedEditing"); // f.VerifyBaselineLinkedEditing(t)
}
