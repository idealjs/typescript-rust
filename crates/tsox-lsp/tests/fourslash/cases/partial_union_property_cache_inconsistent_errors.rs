use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn partial_union_property_cache_inconsistent_errors() {
    let content = r#"// @strict: true
// @lib: esnext
interface ComponentOptions<Props> {
  setup?: (props: Props) => void;
  name?: string;
}

interface FunctionalComponent<P> {
  (props: P): void;
}

type ConcreteComponent<Props> =
  | ComponentOptions<Props>
  | FunctionalComponent<Props>;

type Component<Props = {}> = ConcreteComponent<Props>;

type WithInstallPlugin = { _prefix?: string };


/**/
export function withInstall<C extends Component, T extends WithInstallPlugin>(
  component: C | C[],
  target?: T,
): string {
  const componentWithInstall = (target ?? component) as T;
  const components = Array.isArray(component) ? component : [component];

  const { name } = components[0];
  if (name) {
    return name;
  }

  return "";
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "type C = Component['name']");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
