import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export default function getExePath() {
    const __dirname = path.dirname(fileURLToPath(import.meta.url));
    const normalizedDirname = __dirname.replace(/\\/g, "/");

    let exeDir;
    let binName = "tsgo";

    // Check if we're running from the repo source
    if (normalizedDirname.includes("/npm/lib") || normalizedDirname.includes("/target/")) {
        // Development: use cargo build output
        exeDir = path.resolve(__dirname, "..", "..", "target", "release");
        binName = "tsox"; // Rust binary name
    } else {
        // Installed package: binary is in bin/
        exeDir = path.resolve(__dirname, "..", "bin");
    }

    let exe = path.join(exeDir, binName);
    if (process.platform === "win32") {
        exe += ".exe";
    }

    if (!fs.existsSync(exe)) {
        throw new Error(`Executable not found: ${exe}`);
    }

    return exe;
}
