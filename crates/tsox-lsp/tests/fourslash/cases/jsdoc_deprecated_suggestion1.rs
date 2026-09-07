use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn jsdoc_deprecated_suggestion1() {
    let content = r#"// @experimentalDecorators: true
// @Filename: a.ts
export namespace foo {
    /** @deprecated */
    export function faff () { }
    [|faff|]()
}
const [|a|] = foo.[|faff|]()
foo[[|"faff"|]]
const { [|faff|] } = foo
[|faff|]()
/** @deprecated */
export function bar () {
    foo?.[|faff|]()
}
foo?.[[|"faff"|]]?.()
[|bar|]();
/** @deprecated */
export interface Foo {
    /** @deprecated */
    zzz: number
}
/** @deprecated */
export type QW = [|Foo|][[|"zzz"|]]
export type WQ = [|QW|]
class C {
    /** @deprecated */
    constructor() {
    }
    /** @deprecated */
    m() { }
}
/** @deprecated */
class D {
    constructor() {
    }
}
var c = new [|C|]()
c.[|m|]()
c.[|m|]
new [|D|]()
C
[|D|]
// @Filename: j.tsx
type Props = { someProp?: any }
declare var props: Props
/** @deprecated */
function Compi(_props: Props) {
    return <div></div>
}
[|Compi|];
<[|Compi|] />;
<[|Compi|] {...props}><div></div></[|Compi|]>;
/** @deprecated */
function ttf(_x: unknown) {
}
[|ttf|]` + "`" + `` + "`" + `
[|ttf|]
/** @deprecated */
function dec(_c: unknown) { }
[|dec|]
@[|dec|]
class K { }
// @Filename: b.ts
// imports and aliases
import * as f from './a';
import { [|bar|], [|QW|] } from './a';
f.[|bar|]();
f.foo.[|faff|]();
[|bar|]();
type Z = [|QW|];
type A = f.[|Foo|];
type B = f.[|QW|];
type C = f.WQ;
type [|O|] = Z | A | B | C;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::go_to_file(&mut s, "j.tsx");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
    fourslash::go_to_file(&mut s, "b.ts");
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, []*lsproto.Diagnostic{
}
