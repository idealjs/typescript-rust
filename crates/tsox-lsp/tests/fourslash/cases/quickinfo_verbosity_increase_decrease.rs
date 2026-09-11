use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_increase_decrease() {
    let content = r#"export const JOB_STATES = ["created", "active", "completed", "failed", "retry", "cancelled", "archive"] as const
export type JobState = (typeof JOB_STATES)[number]
type Color = "default" | "primary" | "secondary" | "success" | "warning" | "danger"
const JobsStateToColor/*a*/: Record<
  JobState,
  {
    color: Color
    label: string
    labelPlural: string
  }
> = {
  created: {
    color: "success",
    label: "Направљен",
    labelPlural: "Направљени",
  },
  active: {
    color: "success",
    label: "Активан",
    labelPlural: "Активни",
  },
  completed: {
    color: "success",
    label: "Успешан",
    labelPlural: "Успешни",
  },
  cancelled: {
    color: "default",
    label: "Отаказан",
    labelPlural: "Отаказни",
  },
  failed: {
    color: "danger",
    label: "Пао",
    labelPlural: "Пали",
  },
  archive: {
    color: "default",
    label: "Архивиран",
    labelPlural: "Архивирани",
  },
  retry: {
    color: "warning",
    label: "Понавља се",
    labelPlural: "Понављају се",
  },
}"#;
    let mut s = Session::new_for_test("quickinfoVerbosityIncreaseDecrease", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"a": {0, 1, 0}})
}
