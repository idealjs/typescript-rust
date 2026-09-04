"""Parse tsgo option declarations + enum maps -> JSON metadata for the
differential runner. Mirrors getCompilerVaryByMap exactly."""
import json, re, os

GO = os.path.expanduser("~/workspace/typescript-go")
DECLS = [os.path.join(GO, "tsc/internal/tsoptions", f) for f in
         ("declscompiler.go", "declsbuild.go", "declswatch.go", "declstypeacquisition.go")]
AFFECTS = ("AffectsSemanticDiagnostics", "AffectsEmit", "AffectsModuleResolution",
           "AffectsBindDiagnostics", "AffectsSourceFile", "AffectsDeclarationPath",
           "AffectsBuildInfo", "AffectsProgramStructure")

def parse_decls():
    out = {}
    def fields(block):
        nm = re.search(r'Name:\s*"([^"]+)"', block)
        if not nm: return None
        name = nm.group(1)
        kind = re.search(r'Kind:\s*CommandLineOptionType(\w+)', block)
        kind = kind.group(1) if kind else "String"
        cmdline_only = bool(re.search(r'IsCommandLineOnly:\s*true', block))
        affects = any(re.search(rf'{a}:\s*true', block) for a in AFFECTS)
        map_name = None
        mm = re.search(r'//\s*(\w+Map)', block)
        if mm: map_name = mm.group(1)
        return (name, kind, cmdline_only, affects, map_name)
    def scan_block(src, b):
        depth = 0; j = b
        while True:
            if src[j] == '{': depth += 1
            elif src[j] == '}':
                depth -= 1
                if depth == 0: return j
            j += 1
    for path in DECLS:
        src = open(path).read()
        # array-of-option variables
        for m in re.finditer(r'= \[\]\*CommandLineOption\{', src):
            b = m.end() - 1
            arr_end = scan_block(src, b)
            # walk depth-1 children
            i = b + 1
            while i < arr_end:
                if src[i] == '{':
                    e = scan_block(src, i)
                    f = fields(src[i:e])
                    if f:
                        name, kind, co, aff, mp = f
                        out[name.lower()] = {"kind": kind, "cmdline_only": co,
                                             "affects": aff, "map": mp, "orig": name}
                    i = e + 1
                else:
                    i += 1
        # standalone `var x = &CommandLineOption{...}`
        for m in re.finditer(r'= &CommandLineOption\{', src):
            b = m.end() - 1
            e = scan_block(src, b)
            f = fields(src[b:e])
            if f:
                name, kind, co, aff, mp = f
                out[name.lower()] = {"kind": kind, "cmdline_only": co,
                                     "affects": aff, "map": mp, "orig": name}
    return out

def parse_enum_maps():
    src = open(os.path.join(GO, "tsc/internal/tsoptions/enummaps.go")).read()
    maps = {}
    for m in re.finditer(r'var (\w+Map) = ', src):
        map_name = m.group(1)
        i = src.index('{', m.end())
        depth = 0; j = i
        while True:
            if src[j] == '{': depth += 1
            elif src[j] == '}':
                depth -= 1
                if depth == 0: break
            j += 1
        keys = re.findall(r'Key:\s*"([^"]+)"', src[i:j])
        maps[map_name] = keys
    return maps

if __name__ == "__main__":
    decls = parse_decls()
    maps = parse_enum_maps()
    for name, d in decls.items():
        if d["map"] and d["map"] in maps:
            d["values"] = maps[d["map"]]
        elif d["kind"] == "Boolean":
            d["values"] = ["true", "false"]
    vary = {n: d for n, d in decls.items()
            if d["kind"] in ("Boolean", "Enum") and not d["cmdline_only"] and d["affects"]}
    vary["noemit"] = decls.get("noemit", {"kind": "Boolean", "values": ["true","false"]})
    vary["isolatedmodules"] = decls.get("isolatedmodules", {"kind": "Boolean", "values": ["true","false"]})
    json.dump({"decls": decls, "vary": vary}, open("/tmp/gostd/decls.json", "w"), indent=1)
    print(f"decls={len(decls)} vary_by={len(vary)}")
    print("sample vary:", sorted(vary)[:12])
