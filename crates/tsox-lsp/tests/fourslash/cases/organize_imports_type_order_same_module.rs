use tsox_lsp::fourslash::Session;


#[test]
fn organize_imports_import_kind_order() {
    let content = r#"// @module: commonjs
// @Filename: /main.ts
import { foo } from './package';
import type { Foo } from './package';
import './package';
import Default from './package';
import * as ns from './package';

const x: Foo = foo;
console.log(x, Default, ns);
// @Filename: /package.d.ts
export type Foo = string;
export declare const foo: Foo;
export declare function fn(): void;
export default class Default {}
export as namespace Package;"#;
    let _s = Session::new_for_test("organizeImports_importKindOrder", content);
    // TODO: f.VerifyOrganizeImports(
}

#[test]
fn organize_imports_import_kind_order_multiple_modules() {
    let content = r#"// @module: commonjs
// @Filename: /main.ts
import { b } from './b';
import { a } from './a';
import type { TypeB } from './b';
import type { TypeA } from './a';
import './b';
import './a';

const x: TypeA = a;
const y: TypeB = b;
console.log(x, y);
// @Filename: /a.d.ts
export type TypeA = string;
export declare const a: TypeA;
// @Filename: /b.d.ts
export type TypeB = string;
export declare const b: TypeB;"#;
    let _s = Session::new_for_test("organizeImports_importKindOrderMultipleModules", content);
    // TODO: f.VerifyOrganizeImports(
}
