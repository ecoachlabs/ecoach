import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(SCRIPT_DIR, "..");
const node = process.execPath;
const pnpmCommand = process.platform === "win32" ? "cmd" : "pnpm";
const pnpmArgs = process.platform === "win32" ? ["/c", "pnpm"] : [];

const checks = [
  {
    name: "Frontend production build",
    command: pnpmCommand,
    args: [...pnpmArgs, "--dir", "frontend", "build"],
  },
  {
    name: "IPC command registry coverage",
    command: node,
    args: ["scripts/validate_ipc_command_coverage.mjs"],
  },
  {
    name: "Question SQL schema scan",
    command: node,
    args: ["scripts/validate_question_sql_columns.mjs"],
  },
  {
    name: "Offline runtime contract",
    command: node,
    args: ["scripts/validate_offline_runtime_contract.mjs"],
  },
  {
    name: "Offline dependency scan",
    command: node,
    args: ["scripts/validate_offline_dependency_scan.mjs"],
  },
  {
    name: "Offline capability scan",
    command: node,
    args: ["scripts/validate_offline_capability.mjs"],
  },
  {
    name: "Math CCP content validation",
    command: node,
    args: ["scripts/validate_math_ccp_questions.mjs"],
  },
];

const results = [];

for (const check of checks) {
  console.log(`\n== ${check.name} ==`);
  const result = spawnSync(check.command, check.args, {
    cwd: ROOT,
    encoding: "utf8",
    shell: false,
  });

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) console.error(result.error.message);

  results.push({
    name: check.name,
    exitCode: result.status ?? 1,
  });

  if (result.status !== 0) {
    console.error(`\nOffline readiness failed at: ${check.name}`);
    process.exit(result.status ?? 1);
  }
}

console.log(
  `\n${JSON.stringify(
    {
      status: "ok",
      offline_readiness_checks: results.length,
      cwd: path.relative(ROOT, ROOT) || ".",
    },
    null,
    2,
  )}`,
);
