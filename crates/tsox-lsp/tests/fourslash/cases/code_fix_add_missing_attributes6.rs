use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_attributes6() {
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

const props = { a: 1, b: "", c: [], d: undefined };
const Bar = () =>
    [|<A {...props}></A>|]"#;
    let mut s = Session::new_for_test("codeFixAddMissingAttributes6", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixMissingAttributes")
}
