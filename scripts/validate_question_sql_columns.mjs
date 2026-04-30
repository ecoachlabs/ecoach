import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(SCRIPT_DIR, "..");
const MIGRATIONS_DIR = path.join(ROOT, "migrations", "runtime");
const SOURCE_DIRS = ["src-tauri", "crates"];
const IGNORED_DIRS = new Set([
  ".git",
  "node_modules",
  "target",
  "target-codex-idea32",
  "target-codex-verify",
  "dist",
]);

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function walk(dir, predicate, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (IGNORED_DIRS.has(entry.name)) continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(fullPath, predicate, out);
    } else if (predicate(fullPath)) {
      out.push(fullPath);
    }
  }
  return out;
}

function extractQuestionColumns() {
  const columns = new Set();
  const migrationFiles = walk(
    MIGRATIONS_DIR,
    (filePath) => filePath.toLowerCase().endsWith(".sql"),
  ).sort();

  for (const filePath of migrationFiles) {
    const sql = readText(filePath);
    const createMatch = sql.match(
      /CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+questions\s*\(([\s\S]*?)\n\);/i,
    );
    if (createMatch) {
      for (const rawLine of createMatch[1].split(/\r?\n/)) {
        const line = rawLine.trim().replace(/,$/, "");
        if (!line || /^(CHECK|CONSTRAINT|FOREIGN|PRIMARY|UNIQUE)\b/i.test(line)) {
          continue;
        }
        const columnMatch = line.match(/^["`[]?([A-Za-z_][A-Za-z0-9_]*)["`\]]?\s+/);
        if (columnMatch) {
          columns.add(columnMatch[1]);
        }
      }
    }

    for (const match of sql.matchAll(
      /ALTER\s+TABLE\s+questions\s+ADD\s+COLUMN\s+(?:IF\s+NOT\s+EXISTS\s+)?["`[]?([A-Za-z_][A-Za-z0-9_]*)["`\]]?/gi,
    )) {
      columns.add(match[1]);
    }
  }

  return columns;
}

function lineNumberForOffset(text, offset) {
  return text.slice(0, offset).split(/\r?\n/).length;
}

function validateQuestionColumnReferences(questionColumns) {
  const failures = [];
  const sourceFiles = SOURCE_DIRS.flatMap((dir) =>
    walk(path.join(ROOT, dir), (filePath) => filePath.endsWith(".rs")),
  );

  for (const filePath of sourceFiles) {
    const text = readText(filePath);
    const aliasesQuestions =
      /\b(?:FROM|JOIN)\s+questions\s+(?:AS\s+)?q\b/i.test(text) ||
      /\b(?:FROM|JOIN)\s+questions\s+(?:AS\s+)?q\b/i.test(
        text.replace(/\s+/g, " "),
      );
    const references = [];

    if (aliasesQuestions) {
      for (const match of text.matchAll(/\bq\.([A-Za-z_][A-Za-z0-9_]*)\b/g)) {
        references.push({
          column: match[1],
          offset: match.index ?? 0,
          reference: `q.${match[1]}`,
        });
      }
    }

    for (const reference of references) {
      if (!questionColumns.has(reference.column)) {
        failures.push({
          file: path.relative(ROOT, filePath),
          line: lineNumberForOffset(text, reference.offset),
          reference: reference.reference,
          reason: "column does not exist on the questions table",
        });
      }
    }
  }

  return failures;
}

const questionColumns = extractQuestionColumns();
const failures = validateQuestionColumnReferences(questionColumns);

if (failures.length > 0) {
  console.error(
    JSON.stringify(
      {
        status: "failed",
        checked_table: "questions",
        known_columns: [...questionColumns].sort(),
        failures,
      },
      null,
      2,
    ),
  );
  process.exit(1);
}

console.log(
  JSON.stringify(
    {
      status: "ok",
      checked_table: "questions",
      known_columns: questionColumns.size,
      scanned_sources: SOURCE_DIRS,
    },
    null,
    2,
  ),
);
