import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export default function getExePath() {
    const __dirname = path.dirname(fileURLToPath(import.meta.url));
    const normalizedDirname = __dirname.replace(/\\/g, "/");

    let exeDir;
    let binName = "tsox";

    // Check if we're running from the repo source (dev mode)
    if (normalizedDirname.includes("/npm/lib") || normalizedDirname.includes("/target/")) {
        // Development: use cargo build output
        exeDir = path.resolve(__dirname, "..", "..", "target", "release");
        binName = "tsox";
    } else {
        // Installed package: binary is in bin/
        exeDir = path.resolve(__dirname, "..", "bin");
        // Multi-platform binary naming
        const platform = `${process.platform}-${process.arch}`;
        const platformBin = `tsox-${platform}`;
        if (fs.existsSync(path.join(exeDir, platformBin))) {
            binName = platformBin;
        } else if (fs.existsSync(path.join(exeDir, "tsox"))) {
            binName = "tsox";
        }
    }

    let exe = path.join(exeDir, binName);
    if (process.platform === "win32") {
        exe += ".exe";
    }

    if (!fs.existsSync(exe)) {
        throw new Error(
            `Executable not found: ${exe}\n` +
            `Platform: ${process.platform}-${process.arch}\n` +
            `Please ensure the correct platform binary is installed.`
        );
    }

    return exe;
}
