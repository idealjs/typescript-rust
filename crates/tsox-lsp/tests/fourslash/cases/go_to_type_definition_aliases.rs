use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_aliases() {
    let content = r#"// @Filename: goToTypeDefinitioAliases_module1.ts
interface /*definition*/I {
    p;
}
export {I as I2};
// @Filename: goToTypeDefinitioAliases_module2.ts
import {I2 as I3} from "./goToTypeDefinitioAliases_module1";
var v1: I3;
export {v1 as v2};
// @Filename: goToTypeDefinitioAliases_module3.ts
import {/*reference1*/v2 as v3} from "./goToTypeDefinitioAliases_module2";
/*reference2*/v3;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference1", "reference2")
}
