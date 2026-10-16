use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_non_discriminated_union_shared_prop() {
    let content = r#"// @strict: false
interface Entries {
  /**
   * Plugins info...
   */
  plugins?: Record<string, Record<string, unknown>>;
  /**
   * Output info...
   */
  output?: string;
  /**
   * Format info...
   */
  format?: string;
}

interface Input extends Entries {
  /**
   * Input info...
   */
  input: string;
}

interface Types extends Entries {
  /**
   * Types info...
   */
  types: string;
}

type EntriesOptions = Input | Types;

const options: EntriesOptions[] = [
  {
    input: "./src/index.ts",
    /*1*/output: "./dist/index.mjs",
  },
  {
    types: "./src/types.ts",
    format: "esm",
  },
];"#;
    let mut s = Session::new_for_test("quickInfoJsDocNonDiscriminatedUnionSharedProp", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) Entries.output?: string", "Output info...");
}
