#!/usr/bin/env bash
set -euo pipefail

LIMIT=300
OUT="rs_over_300_lines.csv"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$REPO_ROOT"

scan_rs_file() {
  awk '
    BEGIN { t = 0; out = ""; pend = 0 }
    {
      line = $0
      gsub(/"[^"]*"/, "", line)
      sub(/\/\/.*$/, "", line)
      t += gsub(/#\[test\]/, "", line)
      if (pend && line ~ /^}/) {
        out = out (out == "" ? "" : ";") pendkind "@" pendstart "-" NR
        pend = 0
      }
      while (1) {
        if (line ~ /^pub(\([^)]*\))?[[:space:]]/) { sub(/^pub(\([^)]*\))?[[:space:]]+/, "", line); continue }
        if (line ~ /^async[[:space:]]/) { sub(/^async[[:space:]]+/, "", line); continue }
        if (line ~ /^unsafe[[:space:]]/) { sub(/^unsafe[[:space:]]+/, "", line); continue }
        if (line ~ /^extern([[:space:]]+"[^"]*")?[[:space:]]/) { sub(/^extern([[:space:]]+"[^"]*")?[[:space:]]+/, "", line); continue }
        break
      }
      if (line ~ /^(fn|struct|enum|union|trait|impl|type|const|static|macro_rules!)([[:space:](!:<{]|$)/) {
        kind = line
        sub(/[^a-zA-Z_!].*$/, "", kind)
        if (pend) {
          out = out (out == "" ? "" : ";") pendkind "@" pendstart "-" (NR - 1)
          pend = 0
        }
        if (line ~ /;[[:space:]]*$/ || line ~ /}[[:space:]]*$/) {
          out = out (out == "" ? "" : ";") kind "@" NR "-" NR
        } else {
          pend = 1
          pendkind = kind
          pendstart = NR
        }
      }
    }
    END {
      if (pend) out = out (out == "" ? "" : ";") pendkind "@" pendstart "-" NR
      if (out == "") out = "-"
      print t "|" out
    }
  ' "$1"
}

tmp="$(mktemp)"
find . \
  \( -name target -type d -o -name tests -type d -o -name tests.rs -o -name '*_tests.rs' -o -name '*_generated.rs' \) -prune \
  -o -name '*.rs' -type f -print0 |
while IFS= read -r -d '' file; do
  lines=$(wc -l < "$file")
  out=$(scan_rs_file "$file")
  tcount=${out%%|*}
  items=${out#*|}
  if (( lines > LIMIT )) || [ "$tcount" -gt 0 ]; then
    if [ "$tcount" -gt 0 ]; then
      flag=yes
    else
      flag=no
    fi
    printf '%s,%s,%s,%s,%s\n' "${file#./}" "$lines" "$flag" "$tcount" "$items"
  fi
done | sort -t, -k2,2 -nr > "$tmp"

{
  printf 'file,lines,has_test,test_count,top_level_items\n'
  cat "$tmp"
} > "$OUT"
rm -f "$tmp"

count=$(($(wc -l < "$OUT") - 1))
over_limit=$(awk -F, 'NR>1 && $2 > '"$LIMIT" "$OUT" | wc -l)
with_test=$(grep -c ',yes,' "$OUT" || true)
total_tests=$(awk -F, 'NR>1 { s += $4 } END { print s + 0 }' "$OUT")
no_items=$(grep -c ',-$' "$OUT" || true)
echo "共收录 $count 个文件：超过 $LIMIT 行的 $over_limit 个，含 #[test] 的 $with_test 个（内联测试共 $total_tests 个），$no_items 个第一层无直接声明，结果已写入 $OUT"
