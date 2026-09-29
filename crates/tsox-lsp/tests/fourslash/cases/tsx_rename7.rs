use tsox_lsp::fourslash::Session;


#[test]
fn tsx_rename7() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
interface OptionPropBag {
    [|[|{| "contextRangeIndex": 0 |}propx|]: number|]
    propString: string
    optional?: boolean
}
declare function Opt(attributes: OptionPropBag): JSX.Element;
let opt = <Opt />;
let opt1 = <Opt [|[|{| "contextRangeIndex": 2 |}propx|]={100}|] propString />;
let opt2 = <Opt [|[|{| "contextRangeIndex": 4 |}propx|]={100}|] optional/>;
let opt3 = <Opt wrong />;"#;
    let _s = Session::new_for_test("tsxRename7", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "propx")
}
