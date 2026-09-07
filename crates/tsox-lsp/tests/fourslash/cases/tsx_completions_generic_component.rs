use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completions_generic_component() {
    let content = r#"// @jsx: preserve
// @skipLibCheck: true
// @Filename: file.tsx
 declare namespace JSX {
     interface Element { }
     interface IntrinsicElements {
     }
     interface ElementAttributesProperty { props; }
 }

class Table<P> {
    constructor(public props: P) {}
}

type Props = { widthInCol: number; text: string; };

/**
 * @param width {number} Table width in px
 */
function createTable(width) {
    return <Table<Props> /*1*/ />
}

createTable(800);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
