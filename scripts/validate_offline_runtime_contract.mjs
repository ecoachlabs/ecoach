import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

function read(relativePath) {
  return fs.readFileSync(path.join(ROOT, relativePath), "utf8");
}

function fail(message, examples = []) {
  console.error(message);
  for (const example of examples.slice(0, 10)) console.error(`- ${example}`);
  process.exitCode = 1;
}

const queue = read("frontend/src/ipc/offlineQueue.ts");
const ipc = read("frontend/src/ipc/index.ts");
const policy = read("frontend/src/ipc/offlinePolicy.ts");

const checks = [
  [
    queue.includes("createQueuedCallFingerprint") &&
      /findIndex\(\(call\) => call\.fingerprint ===/.test(queue),
    "Offline queue must deduplicate repeated offline clicks with a stable fingerprint.",
  ],
  [
    queue.includes("fingerprint: string") &&
      queue.includes("existingCallIndex"),
    "Queued IPC calls must persist their fingerprint and update an existing queued call instead of appending duplicates.",
  ],
  [
    queue.includes("normalizeQueuedIpcCall") &&
      queue.includes("createQueuedCallFingerprint(call.command, call.args)"),
    "Offline queue reader must migrate existing queued calls that were stored before fingerprints existed.",
  ],
  [
    ipc.includes("isOfflineCommandQueuedError") &&
      ipc.includes("OfflineCommandQueuedError"),
    "IPC wrapper must expose a helper for screens to identify queued-offline errors.",
  ],
  [
    policy.includes("getOnlineRequiredReason") &&
      policy.includes("web_source") &&
      policy.includes("remote_url"),
    "Command policy must expose the reason an action is online-required.",
  ],
];

for (const [passed, message] of checks) {
  if (!passed) fail(message);
}

if (process.exitCode) process.exit(process.exitCode);

console.log(
  JSON.stringify(
    {
      status: "ok",
      offline_queue_contract: "deduplicated",
      queued_error_helper: true,
      online_required_reason: true,
    },
    null,
    2,
  ),
);
