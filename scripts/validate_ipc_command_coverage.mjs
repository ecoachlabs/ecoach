import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

const frontendCommands = collectFrontendIpcCommands();
const backendCommands = collectBackendCommands();
const missingBackendCommands = [...frontendCommands].filter((command) => !backendCommands.has(command)).sort();

if (missingBackendCommands.length) {
  console.error("Frontend IPC commands missing from the Tauri command registry.");
  for (const command of missingBackendCommands) console.error(`- ${command}`);
  process.exit(1);
}

console.log(
  JSON.stringify(
    {
      status: "ok",
      frontend_ipc_commands: frontendCommands.size,
      backend_registered_commands: backendCommands.size,
      missing_backend_commands: 0,
    },
    null,
    2,
  ),
);

function collectFrontendIpcCommands() {
  const commands = new Set();
  for (const file of collectFiles(path.join(ROOT, "frontend", "src"), /\.(ts|vue)$/)) {
    const text = fs.readFileSync(file, "utf8");
    for (const match of text.matchAll(/\bipc(?:<[^>]+>)?\(\s*['"]([a-zA-Z0-9_]+)['"]/g)) {
      commands.add(match[1]);
    }
  }
  return commands;
}

function collectBackendCommands() {
  const main = fs.readFileSync(path.join(ROOT, "src-tauri", "src", "main.rs"), "utf8");
  const handlerMatch = main.match(/generate_handler!\s*\[([\s\S]*?)\]/);
  if (!handlerMatch) {
    throw new Error("Could not find tauri::generate_handler![] in src-tauri/src/main.rs");
  }

  return new Set(
    [...handlerMatch[1].matchAll(/commands::([a-zA-Z0-9_]+)/g)].map((match) => match[1]),
  );
}

function collectFiles(root, pattern) {
  const output = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (["node_modules", "dist", "target"].includes(entry.name)) continue;

    const target = path.join(root, entry.name);
    if (entry.isDirectory()) output.push(...collectFiles(target, pattern));
    else if (pattern.test(entry.name)) output.push(target);
  }
  return output;
}
