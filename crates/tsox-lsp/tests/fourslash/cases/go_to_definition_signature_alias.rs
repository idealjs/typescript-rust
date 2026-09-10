use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_signature_alias() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
function /*f*/f() {}
const /*g*/g = f;
const /*h*/h = g;
[|/*useF*/f|]();
[|/*useG*/g|]();
[|/*useH*/h|]();
const /*i*/i = () => 0;
const /*iFn*/iFn = function () { return 0; };
const /*j*/j = i;
[|/*useI*/i|]();
[|/*useIFn*/iFn|]();
[|/*useJ*/j|]();
const o = { /*m*/m: () => 0 };
o.[|/*useM*/m|]();
const oFn = { /*mFn*/mFn: function () { return 0; } };
oFn.[|/*useMFn*/mFn|]();
class Component { /*componentCtr*/constructor(props: {}) {} }
type ComponentClass = /*ComponentClass*/new () => Component;
interface ComponentClass2 { /*ComponentClass2*/new(): Component; }

class /*MyComponent*/MyComponent extends Component {}
<[|/*jsxMyComponent*/MyComponent|] />;
new [|/*newMyComponent*/MyComponent|]({});

declare const /*MyComponent2*/MyComponent2: ComponentClass;
<[|/*jsxMyComponent2*/MyComponent2|] />;
new [|/*newMyComponent2*/MyComponent2|]();

declare const /*MyComponent3*/MyComponent3: ComponentClass2;
<[|/*jsxMyComponent3*/MyComponent3|] />;
new [|/*newMyComponent3*/MyComponent3|]();"#;
    let mut s = Session::new_for_test("goToDefinitionSignatureAlias", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "useF", "useG", "useH", "useI", "useIFn", "useJ", "useM", "u
}
