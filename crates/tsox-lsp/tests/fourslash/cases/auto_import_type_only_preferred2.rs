use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn auto_import_type_only_preferred2() {
    let content = r#"// @Filename: /node_modules/react/index.d.ts
export interface ComponentType {}
export interface ComponentProps {}
export declare function useState<T>(initialState: T): [T, (newState: T) => void];
export declare function useEffect(callback: () => void, deps: any[]): void;
// @Filename: /main.ts
import type { ComponentType } from "react";
import { useState } from "react";

export function Component({ prop } : { prop: ComponentType }) {
    const codeIsUnimportant = useState(1);
    useEffect/*1*/(() => {}, []);
}
// @Filename: /main2.ts
import { useState } from "react";
import type { ComponentType } from "react";

type _ = ComponentProps/*2*/;"#;
    let mut s = Session::new_for_test("autoImportTypeOnlyPreferred2", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
