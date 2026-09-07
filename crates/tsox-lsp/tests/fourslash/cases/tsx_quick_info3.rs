use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn tsx_quick_info3() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
interface OptionProp {
    propx: 2
}
class Opt extends React.Component<OptionProp, {}> {
    render() {
        return <div>Hello</div>;
    }
}
const obj1: OptionProp = {
    propx: 2
}
let y1 = <O/*1*/pt pro/*2*/px={2} />;
let y2 = <Opt {...ob/*3*/j1} />;
let y2 = <Opt {...obj1} pr/*4*/opx />;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "class Opt", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) propx: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "const obj1: OptionProp", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(property) propx: true", "")
}
