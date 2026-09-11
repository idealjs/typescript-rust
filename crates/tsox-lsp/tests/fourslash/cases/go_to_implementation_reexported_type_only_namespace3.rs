use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_reexported_type_only_namespace3() {
    let content = r#"
// @Filename: /node_modules/@typescript-eslint/types/index.d.ts
export * as TSESTree from './generated/ast-spec';
export type * as TSESTree from './generated/ast-spec';

// @Filename: /node_modules/@typescript-eslint/types/generated/ast-spec.d.ts
export interface BaseNode {}

// @Filename: /node_modules/@typescript-eslint/utils/index.d.ts
export { TSESTree } from '@typescript-eslint/types';

// @Filename: /src/check-license.ts
import type {TSE/*impl*/STree} from '@typescript-eslint/utils';

let node: TSESTree.Node | undefined;
export default node;
"#;
    let mut s = Session::new_for_test("goToImplementationReexportedTypeOnlyNamespace3", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "impl")
}
