use tsox_lsp::fourslash::Session;


#[test]
fn find_references_jsx_tag_name3() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
namespace JSX {
    export interface Element { }
    export interface IntrinsicElements {
        [|[|/*1*/div|]: any;|]
    }
}

[|const [|/*6*/Comp|] = () =>
    [|<[|/*2*/div|]>
        Some content
        [|<[|/*3*/div|]>More content</[|/*4*/div|]>|]
    </[|/*5*/div|]>|];|]

const x = [|<[|/*7*/Comp|]>
    Content
</[|/*8*/Comp|]>|];"#;
    let _s = Session::new_for_test("findReferencesJSXTagName3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8")
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[5], f.Ranges()[
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[3], f.Ranges()[11], f.Ranges()
}
