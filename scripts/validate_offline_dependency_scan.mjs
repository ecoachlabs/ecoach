import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

const FRONTEND_ROOTS = ["frontend/src"];
const RUST_ROOTS = ["src-tauri/src", "crates"];
const FRONTEND_EXTENSIONS = new Set([".ts", ".vue", ".css", ".html"]);
const RUST_EXTENSIONS = new Set([".rs", ".toml"]);

const allowedRemoteText = [
  /xmlns=/,
  /www\.w3\.org/,
  /schemas\.openxmlformats\.org/,
  /example\.edu/,
  /data:image/,
]

const frontendPatterns = [
  /https?:\/\//,
  /fetch\s*\(/,
  /\baxios\b/,
  /fonts\.googleapis/,
  /fonts\.gstatic/,
  /@import\s+url\(['"]https?:\/\//,
]

const rustPatterns = [
  /\breqwest\b/,
  /\bureq\b/,
  /\bhyper\b/,
  /\bTcpStream\b/,
  /https?:\/\//,
]

const offenders = [
  ...scanRoots(FRONTEND_ROOTS, FRONTEND_EXTENSIONS, frontendPatterns),
  ...scanRoots(RUST_ROOTS, RUST_EXTENSIONS, rustPatterns),
]

if (offenders.length) {
  console.error("Offline dependency scan found remote/network references that need explicit offline policy review.");
  for (const offender of offenders.slice(0, 20)) {
    console.error(`- ${offender.file}:${offender.lineNumber}: ${offender.line.trim()}`);
  }
  process.exit(1);
}

console.log(
  JSON.stringify(
    {
      status: "ok",
      frontend_remote_dependencies: 0,
      rust_network_references: 0,
    },
    null,
    2,
  ),
);

function scanRoots(roots, extensions, patterns) {
  const output = [];
  for (const root of roots) {
    const absoluteRoot = path.join(ROOT, root);
    if (!fs.existsSync(absoluteRoot)) continue;

    for (const file of collectFiles(absoluteRoot, extensions)) {
      const text = fs.readFileSync(file, "utf8");
      const lines = text.split(/\r?\n/);
      lines.forEach((line, index) => {
        if (allowedRemoteText.some((pattern) => pattern.test(line))) return;
        if (!patterns.some((pattern) => pattern.test(line))) return;

        output.push({
          file: path.relative(ROOT, file),
          lineNumber: index + 1,
          line,
        });
      });
    }
  }
  return output;
}

function collectFiles(root, extensions) {
  const output = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (["node_modules", "dist", "target"].includes(entry.name)) continue;

    const target = path.join(root, entry.name);
    if (entry.isDirectory()) {
      output.push(...collectFiles(target, extensions));
    } else if (extensions.has(path.extname(entry.name))) {
      output.push(target);
    }
  }
  return output;
}
