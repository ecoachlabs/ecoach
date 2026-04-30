import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();
const PACK_DIR = path.join(ROOT, "packs", "math-ghana-ccp-b7b9-foundation");

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(PACK_DIR, relativePath), "utf8"));
}

function fail(message, examples) {
  console.error(message);
  if (examples?.length) {
    for (const example of examples.slice(0, 5)) console.error(`- ${example}`);
  }
  process.exitCode = 1;
}

const topics = readJson("curriculum/topics.json").filter((topic) => topic.node_type === "topic");
const questions = readJson("questions/questions.json");
const families = readJson("questions/families.json");
const intelligence = readJson("questions/intelligence.json");

const familyCodes = new Set(families.map((family) => family.family_code));
const topicCounts = new Map();
const failures = {
  optionCorrectCount: [],
  missingFamily: [],
  genericDistractor: [],
  missingMisconception: [],
  missingDetailedExplanation: [],
  missingSolutionBreakdown: [],
  incompleteOptionDiagnostics: [],
  missingLatexMetadata: [],
  invalidInlineLatex: [],
};

for (const question of questions) {
  topicCounts.set(question.topic_code, (topicCounts.get(question.topic_code) ?? 0) + 1);

  if (!familyCodes.has(question.family_code)) {
    failures.missingFamily.push(`${question.topic_code}: ${question.stem}`);
  }

  const correctCount = question.options.filter((option) => option.is_correct).length;
  if (correctCount !== 1) {
    failures.optionCorrectCount.push(`${question.topic_code}: ${question.stem}`);
  }

  if (
    !question.explanation_text?.startsWith("How to solve:") ||
    !question.explanation_text.includes("Step-by-step solution:") ||
    !question.explanation_text.includes("Why each option is right or wrong:") ||
    !question.explanation_text.includes("What each wrong answer reveals:")
  ) {
    failures.missingDetailedExplanation.push(`${question.topic_code}: ${question.stem}`);
  }

  const breakdown = question.solution_breakdown;
  if (
    !breakdown?.commentary ||
    !Array.isArray(breakdown.steps) ||
    breakdown.steps.length < 3 ||
    !breakdown.correct_answer ||
    !Array.isArray(breakdown.option_diagnostics)
  ) {
    failures.missingSolutionBreakdown.push(`${question.topic_code}: ${question.stem}`);
  }

  if (
    !question.latex_stem ||
    !Array.isArray(question.latex_options) ||
    question.latex_options.length !== question.options.length ||
    !question.latex_answer
  ) {
    failures.missingLatexMetadata.push(`${question.topic_code}: ${question.stem}`);
  }

  for (const text of [question.stem, ...question.options.map((option) => option.option_text)]) {
    const open = (String(text).match(/\\\(/g) ?? []).length;
    const close = (String(text).match(/\\\)/g) ?? []).length;
    if (open !== close) failures.invalidInlineLatex.push(`${question.topic_code}: ${text}`);
  }

  if (breakdown?.option_diagnostics?.length !== question.options.length) {
    failures.incompleteOptionDiagnostics.push(`${question.topic_code}: ${question.stem}`);
  }

  for (const option of question.options) {
    if (option.is_correct) continue;
    if (!option.misconception_title) {
      failures.missingMisconception.push(`${question.topic_code} ${option.option_label}: ${question.stem}`);
    }
    if (
      !option.distractor_intent ||
      option.distractor_intent === "Plausible error path for analytics." ||
      !option.distractor_intent.includes("Misstep:") ||
      !option.distractor_intent.includes("Reveals:") ||
      !option.distractor_intent.includes("Needs attention:")
    ) {
      failures.genericDistractor.push(`${question.topic_code} ${option.option_label}: ${question.stem}`);
    }
  }
}

const badTopicCounts = topics
  .map((topic) => [topic.code, topicCounts.get(topic.code) ?? 0])
  .filter(([, count]) => count !== 50);

if (questions.length !== 2850) fail(`Expected 2850 questions, found ${questions.length}.`);
if (families.length !== 342) fail(`Expected 342 families, found ${families.length}.`);
if (intelligence.length !== questions.length) {
  fail(`Expected ${questions.length} intelligence records, found ${intelligence.length}.`);
}
if (badTopicCounts.length) {
  fail(
    "Expected exactly 50 questions per content topic.",
    badTopicCounts.map(([code, count]) => `${code}: ${count}`),
  );
}

for (const [key, examples] of Object.entries(failures)) {
  if (examples.length) fail(`${key}: ${examples.length} failing records.`, examples);
}

if (process.exitCode) process.exit(process.exitCode);

console.log(
  JSON.stringify(
    {
      status: "ok",
      content_topics: topics.length,
      questions: questions.length,
      families: families.length,
      intelligence: intelligence.length,
      questions_per_topic: 50,
      wrong_options_checked: questions.length * 3,
    },
    null,
    2,
  ),
);
