import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

function read(relativePath) {
  return fs.readFileSync(path.join(ROOT, relativePath), "utf8");
}

function exists(relativePath) {
  return fs.existsSync(path.join(ROOT, relativePath));
}

function fail(message, examples = []) {
  console.error(message);
  for (const example of examples.slice(0, 8)) console.error(`- ${example}`);
  process.exitCode = 1;
}

const requiredFiles = [
  "frontend/src/stores/connectivity.ts",
  "frontend/src/ipc/offlinePolicy.ts",
  "frontend/src/ipc/offlineQueue.ts",
  "frontend/src/components/system/OfflineStatusBanner.vue",
];

const missingFiles = requiredFiles.filter((file) => !exists(file));
if (missingFiles.length) fail("Missing offline capability files.", missingFiles);

if (!process.exitCode) {
  const connectivity = read("frontend/src/stores/connectivity.ts");
  const policy = read("frontend/src/ipc/offlinePolicy.ts");
  const queue = read("frontend/src/ipc/offlineQueue.ts");
  const ipc = read("frontend/src/ipc/index.ts");
  const app = read("frontend/src/App.vue");

  const checks = [
    [
      connectivity.includes("window.addEventListener('online'") &&
        connectivity.includes("window.addEventListener('offline'"),
      "Connectivity store must listen for browser online/offline events.",
    ],
    [
      connectivity.includes("probeConnectivity") && connectivity.includes("connectivity-restored"),
      "Connectivity store must expose a connectivity probe and restoration event.",
    ],
    [
      policy.includes("offline-native") &&
        policy.includes("online-enhanced") &&
        policy.includes("online-required"),
      "Command policy must define offline-native, online-enhanced, and online-required modes.",
    ],
    [
      policy.includes("register_curriculum_source") &&
        policy.includes("run_foundry_job") &&
        policy.includes("check_entitlement"),
      "Command policy must classify currently risky/admin/network-adjacent commands.",
    ],
    [
      queue.includes("localStorage") &&
        queue.includes("enqueueOfflineCall") &&
        queue.includes("flushOfflineQueue"),
      "Offline queue must persist calls locally and expose enqueue/flush functions.",
    ],
    [
      ipc.includes("getCommandPolicy") &&
        ipc.includes("enqueueOfflineCall") &&
        ipc.includes("flushQueuedIpcCalls") &&
        ipc.includes("startOfflineQueueAutoFlush"),
      "IPC wrapper must apply command policy and auto-flush queued work.",
    ],
    [
      app.includes("OfflineStatusBanner") && app.includes("startMonitoring"),
      "App root must initialize connectivity monitoring and render offline status.",
    ],
  ];

  for (const [passed, message] of checks) {
    if (!passed) fail(message);
  }

  const sourceTexts = [
    ["frontend/src", collectFiles(path.join(ROOT, "frontend", "src"))],
    ["frontend/dist", exists("frontend/dist") ? collectFiles(path.join(ROOT, "frontend", "dist")) : []],
  ];
  for (const [label, files] of sourceTexts) {
    const offenders = [];
    for (const file of files) {
      const text = fs.readFileSync(file, "utf8");
      if (/fonts\.googleapis|fonts\.gstatic|@import\s+url\(['"]https?:\/\//.test(text)) {
        offenders.push(path.relative(ROOT, file));
      }
    }
    if (offenders.length) fail(`${label} contains remote font/import dependencies.`, offenders);
  }
}

if (process.exitCode) process.exit(process.exitCode);

console.log(
  JSON.stringify(
    {
      status: "ok",
      offline_capability_files: requiredFiles.length,
      remote_font_imports: 0,
    },
    null,
    2,
  ),
);

function collectFiles(root) {
  const output = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const target = path.join(root, entry.name);
    if (entry.isDirectory()) output.push(...collectFiles(target));
    else if (/\.(ts|vue|js|css|html)$/.test(entry.name)) output.push(target);
  }
  return output;
}
