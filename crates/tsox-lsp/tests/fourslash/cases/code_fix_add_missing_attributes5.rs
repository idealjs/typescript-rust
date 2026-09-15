use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_add_missing_attributes5() {
    let content = r#"// @jsx: preserve
// @filename: foo.tsx
interface P {
    a: number;
    b: string;
    c: number[];
    d: any;
}

const A = ({ a, b, c, d }: P) =>
    <div>{a}{b}{c}{d}</div>;

const Bar = () =>
    [|<A a={100} b={""} c={[]} d={undefined}></A>|]"#;
    let _s = Session::new_for_test("codeFixAddMissingAttributes5", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingAttributes")
}
