import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();
const PACK_DIR = path.join(ROOT, "packs", "math-ghana-ccp-b7b9-foundation");
const DOC_SEED = path.join(
  ROOT,
  "docs",
  "curriculum",
  "math_ghana_ccp_b7_b9",
  "mathematics_ccp_seed_package.json",
);

const QUESTION_TARGET_PER_TOPIC = 50;
const SEED_BATCH = "math-ccp-b7b9-analytics-v1";

const FAMILY_BLUEPRINT = [
  {
    key: "FOUNDATION",
    name: "Foundation Checks",
    count: 8,
    familyType: "recurring_pattern",
    knowledgeRole: "key_concept",
    cognitiveDemand: "comprehension",
    solvePattern: "direct_retrieval",
    pedagogicFunction: "foundation_check",
  },
  {
    key: "WORKED",
    name: "Worked Pattern",
    count: 8,
    familyType: "worked_example_template",
    knowledgeRole: "worked_example",
    cognitiveDemand: "application",
    solvePattern: "substitute_and_solve",
    pedagogicFunction: "foundation_check",
  },
  {
    key: "TRANSFER",
    name: "Transfer Checks",
    count: 9,
    familyType: "recurring_pattern",
    knowledgeRole: "application",
    cognitiveDemand: "application",
    solvePattern: "multi_step_reasoning",
    pedagogicFunction: "transfer_check",
  },
  {
    key: "MISCONCEPTION",
    name: "Misconception Probes",
    count: 9,
    familyType: "misconception_cluster",
    knowledgeRole: "comparison",
    cognitiveDemand: "analysis",
    solvePattern: "multi_step_reasoning",
    pedagogicFunction: "misconception_diagnosis",
  },
  {
    key: "EXAM",
    name: "Exam Patterns",
    count: 8,
    familyType: "exam_structure",
    knowledgeRole: "procedure",
    cognitiveDemand: "analysis",
    solvePattern: "pattern_spotting",
    pedagogicFunction: "exam_pattern_familiarization",
  },
  {
    key: "SPEED",
    name: "Speed Builders",
    count: 8,
    familyType: "recurring_pattern",
    knowledgeRole: "procedure",
    cognitiveDemand: "recall",
    solvePattern: "direct_retrieval",
    pedagogicFunction: "speed_build",
  },
];

const GENERATED_MISCONCEPTION_BLUEPRINT = [
  {
    titleSuffix: "Surface Value Copy",
    cause_type: "surface_similarity",
    wrong_answer_pattern: "surface_pattern_copy",
    statement: "Chooses an answer because it repeats a visible number, keyword, or shape of the question without completing the required mathematical step.",
    correction_hint: "Pause to identify what is being asked, list the given values, then perform the needed operation before comparing options.",
    severity: 5800,
  },
  {
    titleSuffix: "Wrong Operation Or Formula",
    cause_type: "step_confusion",
    wrong_answer_pattern: "formula_selection_error",
    statement: "Uses a related operation, inverse operation, or familiar formula in the wrong context.",
    correction_hint: "Match the operation or formula to the condition in the stem before substituting values.",
    severity: 7000,
  },
  {
    titleSuffix: "Notation Or Representation Misread",
    cause_type: "language_confusion",
    wrong_answer_pattern: "concept_label_confusion",
    statement: "Misreads notation, labels, units, signs, powers, coordinates, fractions, or graph information.",
    correction_hint: "Translate the notation into words and check units, signs, labels, and order before calculating.",
    severity: 6500,
  },
];

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(PACK_DIR, relativePath), "utf8"));
}

function writeJson(relativePath, value) {
  fs.writeFileSync(
    path.join(PACK_DIR, relativePath),
    `${JSON.stringify(value, null, 2)}\n`,
  );
}

function slugCode(value) {
  return value.replace(/[^A-Za-z0-9]+/g, "_").replace(/^_|_$/g, "");
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function byTopic(records) {
  const map = new Map();
  for (const record of records) {
    const topicCode = record.topic_code ?? record.topicCode;
    if (!topicCode) continue;
    if (!map.has(topicCode)) map.set(topicCode, []);
    map.get(topicCode).push(record);
  }
  return map;
}

function pick(items, index, fallback) {
  if (!items || items.length === 0) return fallback;
  return items[index % items.length];
}

function formatNumber(value) {
  if (Number.isInteger(value)) return String(value);
  return String(Number(value.toFixed(3)));
}

function gcd(left, right) {
  let a = Math.abs(left);
  let b = Math.abs(right);
  while (b !== 0) {
    const next = a % b;
    a = b;
    b = next;
  }
  return a || 1;
}

function fraction(numerator, denominator) {
  const divisor = gcd(numerator, denominator);
  const n = numerator / divisor;
  const d = denominator / divisor;
  return d === 1 ? String(n) : `${n}/${d}`;
}

function uniqueOptions(answer, distractors) {
  const seen = new Set([String(answer)]);
  const options = [String(answer)];
  for (const distractor of [...distractors, ...fallbackDistractors(answer)]) {
    const text = String(distractor);
    if (!text || seen.has(text)) continue;
    seen.add(text);
    options.push(text);
    if (options.length === 4) break;
  }
  let pad = 1;
  while (options.length < 4) {
    const text = `Uses an unrelated rule (${pad})`;
    if (!seen.has(text)) {
      seen.add(text);
      options.push(text);
    }
    pad += 1;
  }
  return options;
}

function fallbackDistractors(answer) {
  const text = String(answer);
  const fractionMatch = text.match(/^(-?\d+)\/(\d+)$/);
  if (fractionMatch) {
    const n = Number(fractionMatch[1]);
    const d = Number(fractionMatch[2]);
    return [`${n + 1}/${d}`, `${n}/${d + 1}`, `${d}/${n}`];
  }

  const coordinateMatch = text.match(/^\((-?\d+),\s*(-?\d+)\)$/);
  if (coordinateMatch) {
    const x = Number(coordinateMatch[1]);
    const y = Number(coordinateMatch[2]);
    return [`(${-x}, ${y})`, `(${x}, ${-y})`, `(${y}, ${x})`];
  }

  const numericMatch = text.match(/^(-?\d+(?:\.\d+)?)(.*)$/);
  if (numericMatch) {
    const value = Number(numericMatch[1]);
    const suffix = numericMatch[2] ?? "";
    return [
      `${formatNumber(value + 1)}${suffix}`,
      `${formatNumber(value - 1)}${suffix}`,
      `${formatNumber(value * 2)}${suffix}`,
    ];
  }

  return [
    "Uses the wrong operation",
    "Matches a different topic",
    "Skips the required condition",
  ];
}

function rotateOptions(optionTexts, correctText, shift) {
  const rotated = optionTexts.map((text, index) => ({
    option_text: text,
    is_correct: text === correctText,
    original_position: index,
  }));
  const offset = shift % rotated.length;
  const ordered = [...rotated.slice(offset), ...rotated.slice(0, offset)];
  return ordered.map((option, index) => ({
    option_label: String.fromCharCode(65 + index),
    option_text: option.option_text,
    is_correct: option.is_correct,
    original_position: option.original_position,
    position: index,
  }));
}

function ensureTopicMisconceptions(contentTopics, records) {
  const baseRecords = records.filter((record) => record.generated_by !== SEED_BATCH);
  const output = [...baseRecords];
  const existingByTopic = byTopic(baseRecords);

  for (const topic of contentTopics) {
    const ensured = [...(existingByTopic.get(topic.code) ?? [])];
    const titles = new Set(ensured.map((record) => record.title));

    for (const blueprint of GENERATED_MISCONCEPTION_BLUEPRINT) {
      if (ensured.length >= 3) break;
      const title = `${topic.code} Misstep: ${blueprint.titleSuffix}`;
      if (titles.has(title)) continue;

      const record = {
        topic_code: topic.code,
        title,
        misconception_statement: blueprint.statement,
        cause_type: blueprint.cause_type,
        wrong_answer_pattern: blueprint.wrong_answer_pattern,
        correction_hint: blueprint.correction_hint,
        severity: blueprint.severity,
        generated_by: SEED_BATCH,
      };
      output.push(record);
      ensured.push(record);
      titles.add(title);
    }
  }

  return output;
}

function mapOutsideLatex(text, mapper) {
  const parts = String(text).split(/(\\\(.+?\\\)|\\\[.+?\\\])/gs);
  return parts
    .map((part) => {
      if (part.startsWith("\\(") || part.startsWith("\\[")) return part;
      return mapper(part);
    })
    .join("");
}

function toLatexExpression(expression) {
  return String(expression)
    .replace(/\bsqrt\(([^)]+)\)/gi, "\\sqrt{$1}")
    .replace(/(-?\d+)\s*\/\s*(-?\d+)/g, "\\frac{$1}{$2}")
    .replace(/\b([A-Za-z]|\d+(?:\.\d+)?)\^(-?\d+)/g, "$1^{$2}")
    .replace(/>=/g, "\\ge ")
    .replace(/<=/g, "\\le ")
    .replace(/\*/g, "\\times ")
    .replace(/°/g, "^\\circ")
    .replace(/\s+/g, " ")
    .trim();
}

function mathifyText(text) {
  const mathPattern = /sqrt\([^)]+\)|\(-?\d+(?:\.\d+)?,\s*-?\d+(?:\.\d+)?\)|\b-?\d+\s*:\s*-?\d+\b|\b-?\d+\s*\/\s*-?\d+\b|\b(?:[A-Za-z]|\d+(?:\.\d+)?)\^-?\d+\b|\b[A-Za-z]\s*(?:[+\-*/=<>]|>=|<=)\s*-?\d+(?:\.\d+)?(?:\s*(?:[+\-*/=<>]|>=|<=)\s*-?\d+(?:\.\d+)?)*\b|\b-?\d+(?:\.\d+)?\s*(?:[+\-*/=<>]|>=|<=)\s*-?\d+(?:\.\d+)?(?:\s*(?:[+\-*/=<>]|>=|<=)\s*-?\d+(?:\.\d+)?)*\b|\b-?\d+(?:\.\d+)?\s*%|\b-?\d+(?:\.\d+)?\s*(?:cm|m|km|kg|litres?|units?|degrees?)\b/gi;

  return mapOutsideLatex(text, (plain) =>
    plain
      .replace(/(\d)\s+x\s+(?=[\d(])/gi, "$1 \\(\\times\\) ")
      .replace(mathPattern, (match) => `\\(${toLatexExpression(match)}\\)`),
  );
}

function misconceptionForOption(topicMisconceptions, wrongIndex) {
  const fallback = GENERATED_MISCONCEPTION_BLUEPRINT[wrongIndex % GENERATED_MISCONCEPTION_BLUEPRINT.length];
  return (
    topicMisconceptions[wrongIndex % topicMisconceptions.length] ?? {
      title: `Generated diagnostic: ${fallback.titleSuffix}`,
      misconception_statement: fallback.statement,
      correction_hint: fallback.correction_hint,
      cause_type: fallback.cause_type,
      wrong_answer_pattern: fallback.wrong_answer_pattern,
    }
  );
}

function defaultRevealedDifficulty(misconception, wrongIndex) {
  const pattern = misconception?.wrong_answer_pattern ?? "";
  if (pattern.includes("formula")) return "Selecting the operation or formula before substituting values.";
  if (pattern.includes("concept") || pattern.includes("label")) return "Reading notation, units, labels, and representations accurately.";
  if (pattern.includes("inverse")) return "Choosing the inverse step and preserving equality.";
  return [
    "Separating the given information from the quantity being asked for.",
    "Completing the calculation instead of matching a visible number.",
    "Checking the final option against the condition in the stem.",
  ][wrongIndex % 3];
}

function buildWrongOptionDiagnostic(option, correctAnswer, topic, task, misconception, wrongIndex) {
  const statement = misconception?.misconception_statement ?? "This option follows a predictable but incorrect reasoning path.";
  const revealedDifficulty = defaultRevealedDifficulty(misconception, wrongIndex);
  const remediationFocus =
    misconception?.correction_hint ??
    "Restate the question, choose the correct method, calculate carefully, and compare the result with the options.";
  const misstep = `${statement} The learner may arrive at ${option.option_text} instead of ${correctAnswer} by using that shortcut or incorrect step.`;

  return {
    misconception_title: misconception?.title ?? `${topic.code} Misstep ${wrongIndex + 1}`,
    distractor_intent: `Misstep: ${misstep} Reveals: ${revealedDifficulty} Needs attention: ${remediationFocus}`,
    diagnostic_note: `${option.option_text} is a diagnostic wrong answer for ${topic.code}; it is designed to expose ${revealedDifficulty.toLowerCase()}`,
    revealed_difficulty: revealedDifficulty,
    remediation_focus: remediationFocus,
    rationale: `Option ${option.option_label} is not correct because it gives ${option.option_text}, while the required result is ${correctAnswer}. The likely error is: ${statement}`,
    misconception_code: misconceptionCode(misconception, wrongIndex),
  };
}

function attachOptionDiagnostics(options, topic, topicMisconceptions, task) {
  let wrongIndex = 0;
  return options.map((option) => {
    const latexText = mathifyText(option.option_text);
    if (option.is_correct) {
      return {
        ...option,
        option_text: latexText,
        diagnostic_note: `Correct answer. This option follows the intended method and satisfies the condition in ${topic.code}.`,
      };
    }

    const misconception = misconceptionForOption(topicMisconceptions, wrongIndex);
    const diagnostic = buildWrongOptionDiagnostic(
      option,
      task.answer,
      topic,
      task,
      misconception,
      wrongIndex,
    );
    wrongIndex += 1;

    return {
      ...option,
      option_text: latexText,
      misconception_title: diagnostic.misconception_title,
      distractor_intent: diagnostic.distractor_intent,
      diagnostic_note: diagnostic.diagnostic_note,
      revealed_difficulty: diagnostic.revealed_difficulty,
      remediation_focus: diagnostic.remediation_focus,
      misconception_code: diagnostic.misconception_code,
      rationale: diagnostic.rationale,
    };
  });
}

function topicLevel(topic) {
  return topic.code.split(".")[0] || "B7";
}

function gradeDifficultyOffset(topic) {
  if (topic.difficulty_band === "easy") return 0;
  if (topic.difficulty_band === "hard") return 1600;
  if (topic.difficulty_band === "advanced") return 2400;
  return 850;
}

function makeDifficulty(topic, slot, family) {
  const familyOffset = {
    FOUNDATION: 0,
    WORKED: 500,
    TRANSFER: 900,
    MISCONCEPTION: 1200,
    EXAM: 1500,
    SPEED: 300,
  }[family.key];
  return clamp(3000 + gradeDifficultyOffset(topic) + familyOffset + (slot % 5) * 250, 2500, 9200);
}

function makeSourceRef(meta, topic) {
  const anchors = meta?.source_anchors?.length ? meta.source_anchors.join(",") : "curriculum";
  return `Ghana NaCCA CCP Mathematics 2020:${topic.code}:${anchors}`;
}

function makeExplanation(topic, skill, objective, task, meta) {
  const anchorText = meta?.source_anchors?.length
    ? ` Source anchors: ${meta.source_anchors.join(", ")}.`
    : "";
  const skillText = skill?.canonical_title ? ` Skill node: ${skill.canonical_title}.` : "";
  const objectiveText = objective?.objective_text ? ` Objective: ${objective.objective_text}.` : "";
  return `${task.explanation} This question belongs to ${topic.code}: ${topic.name}.${objectiveText}${skillText}${anchorText}`;
}

function buildSolutionBreakdown(topic, meta, skill, objective, task, options) {
  const correctOption = options.find((option) => option.is_correct);
  const correctAnswer = String(task.answer);
  const objectiveText = objective?.objective_text ?? topic.name;
  const skillText = skill?.canonical_title ?? topic.name;
  const anchorText = meta?.source_anchors?.length
    ? meta.source_anchors.join(", ")
    : "Ghana NaCCA CCP Mathematics curriculum";

  const commentary = mathifyText(
    `Start by reading the command in the stem, identifying the given information, and deciding which ${topic.name} method is needed before looking at the options.`,
  );
  const steps = [
    `Read the question as a ${topic.code} item and restate the target: ${objectiveText}.`,
    `Identify the relevant skill: ${skillText}. This tells you which rule, formula, representation, or calculation path to use.`,
    `Carry out the intended method: ${task.explanation}`,
    `Compare the result with the options and select ${correctOption?.option_label ?? "the correct option"}, which gives ${correctAnswer}.`,
  ].map(mathifyText);

  const optionDiagnostics = options.map((option) => {
    if (option.is_correct) {
      return {
        label: option.option_label,
        text: option.option_text,
        latex: option.option_text,
        is_correct: true,
        rationale: mathifyText(
          `This option is correct because it matches the result ${correctAnswer} after the required mathematical step is completed.`,
        ),
        revealed_difficulty: null,
        remediation_focus: null,
        misconception_title: null,
      };
    }

    return {
      label: option.option_label,
      text: option.option_text,
      latex: option.option_text,
      is_correct: false,
      rationale: mathifyText(option.rationale),
      misconception_title: option.misconception_title,
      revealed_difficulty: option.revealed_difficulty,
      remediation_focus: option.remediation_focus,
      misconception_code: option.misconception_code,
    };
  });

  return {
    commentary,
    steps,
    correct_answer: correctAnswer,
    correct_option_label: correctOption?.option_label,
    latex_answer: mathifyText(correctAnswer),
    option_diagnostics: optionDiagnostics,
    curriculum_link: {
      topic_code: topic.code,
      topic_name: topic.name,
      objective_text: objectiveText,
      skill_title: skillText,
      source_anchors: anchorText,
    },
  };
}

function buildDetailedExplanation(topic, meta, skill, objective, task, solutionBreakdown) {
  const correctOption = solutionBreakdown.correct_option_label ?? "the correct option";
  const correctAnswer = solutionBreakdown.correct_answer;
  const optionLines = solutionBreakdown.option_diagnostics
    .map((diagnostic) => {
      if (diagnostic.is_correct) {
        return `${diagnostic.label}: correct because it gives ${correctAnswer} after the intended method.`;
      }
      return `${diagnostic.label}: wrong because ${diagnostic.rationale} Likely misstep: ${diagnostic.misconception_title}.`;
    })
    .join(" ");
  const revealLines = solutionBreakdown.option_diagnostics
    .filter((diagnostic) => !diagnostic.is_correct)
    .map(
      (diagnostic) =>
        `${diagnostic.label}: reveals ${diagnostic.revealed_difficulty}; needs attention on ${diagnostic.remediation_focus}`,
    )
    .join(" ");
  const anchorText = meta?.source_anchors?.length
    ? ` Source anchors: ${meta.source_anchors.join(", ")}.`
    : "";
  const skillText = skill?.canonical_title ? ` Skill focus: ${skill.canonical_title}.` : "";
  const objectiveText = objective?.objective_text ? ` Objective: ${objective.objective_text}.` : "";

  return mathifyText(
    [
      `How to solve: ${solutionBreakdown.commentary}`,
      `Step-by-step solution: ${solutionBreakdown.steps.join(" ")}`,
      `Why each option is right or wrong: ${optionLines}`,
      `What each wrong answer reveals: ${revealLines}`,
      `Curriculum link: ${topic.code} ${topic.name}.${objectiveText}${skillText}${anchorText} The correct choice is ${correctOption}, giving ${correctAnswer}.`,
      `Original solution note: ${task.explanation}`,
    ].join("\n\n"),
  );
}

function nearbyTopicName(allTopics, topic, slot) {
  const sameGrade = allTopics.filter((candidate) => topicLevel(candidate) === topicLevel(topic));
  return pick(sameGrade, slot + 3, topic).name;
}

function buildTask(topic, objective, slot, family, allTopics) {
  const name = topic.name.toLowerCase();
  const grade = topicLevel(topic);
  const nearby = nearbyTopicName(allTopics, topic, slot);

  if (name.includes("standard form")) return standardFormTask(slot);
  if (name.includes("place value") || name.includes("round")) return roundingTask(slot);
  if (name.includes("compare and order")) return compareOrderTask(slot);
  if (name.includes("mental mathematics")) return mentalMathTask(slot);
  if (name.includes("powers") || name.includes("indices")) return powersTask(slot);
  if (name.includes("surds")) return surdsTask(slot);
  if (name.includes("sets") || name.includes("rational number system")) return setsTask(slot);
  if (name.includes("fraction") && (name.includes("multiply") || name.includes("dividing") || name.includes("operation"))) return fractionOperationTask(slot);
  if (name.includes("fraction") && (name.includes("addition") || name.includes("subtraction"))) return fractionAddTask(slot);
  if (name.includes("fraction")) return fractionSimplifyTask(slot);
  if (name.includes("ratio") || name.includes("rate") || name.includes("proportion")) return ratioTask(slot);
  if (name.includes("simultaneous")) return simultaneousTask(slot);
  if (name.includes("inequalit")) return inequalityTask(slot);
  if (name.includes("linear equation")) return equationTask(slot);
  if (name.includes("algebraic") || name.includes("factorise") || name.includes("change of subject")) return algebraTask(slot);
  if (name.includes("linear relation") || name.includes("table of values") || name.includes("gradient")) return linearRelationTask(slot);
  if (name.includes("surface area") || name.includes("prisms")) return surfaceAreaTask(slot);
  if (name.includes("pythagoras") || name.includes("trigonometric") || name.includes("area of a circle")) return pythagorasTask(slot);
  if (name.includes("perimeter") || name.includes("circumference")) return perimeterTask(slot);
  if (name.includes("area of a triangle")) return triangleAreaTask(slot);
  if (name.includes("bearing") || name.includes("vector")) return vectorTask(slot);
  if (name.includes("enlargement")) return enlargementTask(slot);
  if (name.includes("reflection") || name.includes("translation") || name.includes("rotation") || name.includes("transformation")) return transformationTask(slot, name);
  if (name.includes("construct")) return constructionTask(slot);
  if (name.includes("angle") || name.includes("parallel lines") || name.includes("polygon")) return angleTask(slot);
  if (name.includes("central tendency") || name.includes("mean") || name.includes("median") || name.includes("mode")) return centralTendencyTask(slot);
  if (name.includes("probabilit") || name.includes("sample space")) return probabilityTask(slot, name);
  if (name.includes("data") || name.includes("frequency") || name.includes("histogram") || name.includes("graph")) return dataTask(slot);
  if (name.includes("addition") || name.includes("subtraction") || name.includes("multiplication") || name.includes("division")) return operationsTask(slot);

  return curriculumFitTask(topic, objective, family, grade, nearby);
}

function roundingTask(slot) {
  const places = [10, 100, 1000, 10000, 100000];
  const place = places[slot % places.length];
  const number = 1234567893 + slot * 98765;
  const answer = Math.round(number / place) * place;
  return {
    stem: `Round ${number} to the nearest ${place}.`,
    answer: formatNumber(answer),
    distractors: [
      formatNumber(Math.floor(number / place) * place),
      formatNumber(Math.ceil(number / place) * place),
      formatNumber(answer + place),
      formatNumber(number),
    ],
    explanation: `Rounding to the nearest ${place} means checking the digit immediately to the right of that place value.`,
    solvePattern: "substitute_and_solve",
  };
}

function standardFormTask(slot) {
  const base = [3200000, 48000, 70500000, 0.0046][slot % 4];
  const answers = ["3.2 x 10^6", "4.8 x 10^4", "7.05 x 10^7", "4.6 x 10^-3"];
  return {
    stem: `Write ${base} in standard form.`,
    answer: answers[slot % 4],
    distractors: ["32 x 10^5", "0.32 x 10^7", "3.2 x 10^5", "4.6 x 10^3"],
    explanation: "Standard form writes a number as a value from 1 up to 10 multiplied by a power of 10.",
    solvePattern: "pattern_spotting",
  };
}

function compareOrderTask(slot) {
  const a = 1000000000 + slot * 1103;
  const b = a + 7000;
  const c = a - 9000;
  const answer = `${c}, ${a}, ${b}`;
  return {
    stem: `Arrange these numbers in ascending order: ${a}, ${b}, ${c}.`,
    answer,
    distractors: [`${b}, ${a}, ${c}`, `${a}, ${c}, ${b}`, `${c}, ${b}, ${a}`],
    explanation: "Ascending order starts with the smallest value and ends with the largest value.",
    solvePattern: "direct_retrieval",
  };
}

function mentalMathTask(slot) {
  const a = 25 + slot;
  const b = 4;
  const c = 3 + (slot % 7);
  const answer = a * (b + c);
  return {
    stem: `Use the distributive property to find ${a} x (${b} + ${c}).`,
    answer: formatNumber(answer),
    distractors: [formatNumber(a * b + c), formatNumber(a + b * c), formatNumber((a + b) * c)],
    explanation: `Compute ${a} x ${b} and ${a} x ${c}, then add the products.`,
    solvePattern: "substitute_and_solve",
  };
}

function operationsTask(slot) {
  const a = 240 + slot * 7;
  const b = 18 + (slot % 9);
  const answer = a + b * 3;
  return {
    stem: `A shop sold ${a} exercise books in the morning and ${b} packs of 3 in the afternoon. How many exercise books were sold in all?`,
    answer: formatNumber(answer),
    distractors: [formatNumber(a + b), formatNumber((a + b) * 3), formatNumber(a - b * 3)],
    explanation: "Multiply the afternoon packs by 3, then add the morning sales.",
    solvePattern: "multi_step_reasoning",
  };
}

function powersTask(slot) {
  const base = 2 + (slot % 4);
  const power = 2 + (slot % 3);
  const answer = base ** power;
  return {
    stem: `Evaluate ${base}^${power}.`,
    answer: formatNumber(answer),
    distractors: [formatNumber(base * power), formatNumber(base + power), formatNumber(base ** (power + 1))],
    explanation: `${base}^${power} means multiply ${base} by itself ${power} times.`,
    solvePattern: "direct_retrieval",
  };
}

function surdsTask(slot) {
  const pairs = [
    ["sqrt(50)", "5sqrt(2)", "25sqrt(2)", "10sqrt(5)", "2sqrt(25)"],
    ["2sqrt(3) + 5sqrt(3)", "7sqrt(3)", "10sqrt(3)", "7sqrt(6)", "3sqrt(7)"],
    ["sqrt(72)", "6sqrt(2)", "36sqrt(2)", "8sqrt(9)", "2sqrt(18)"],
  ];
  const [stemValue, answer, ...distractors] = pairs[slot % pairs.length];
  return {
    stem: `Simplify ${stemValue}.`,
    answer,
    distractors,
    explanation: "Simplify surds by taking out the largest square factor, then combine only like surds.",
    solvePattern: "pattern_spotting",
  };
}

function setsTask(slot) {
  const a = 18 + (slot % 8);
  const b = 15 + (slot % 6);
  const both = 5 + (slot % 4);
  const answer = a + b - both;
  return {
    stem: `In a class, ${a} learners like football, ${b} like athletics, and ${both} like both. How many like football or athletics?`,
    answer: formatNumber(answer),
    distractors: [formatNumber(a + b), formatNumber(a + b + both), formatNumber(a - both)],
    explanation: "For two sets, add both group counts and subtract the overlap once.",
    solvePattern: "multi_step_reasoning",
  };
}

function fractionSimplifyTask(slot) {
  const numerator = 6 + (slot % 8) * 2;
  const denominator = numerator * 2;
  const answer = fraction(numerator, denominator);
  return {
    stem: `Write ${numerator}/${denominator} in simplest form.`,
    answer,
    distractors: [`${numerator / 2}/${denominator}`, `${denominator}/${numerator}`, `${numerator}/${denominator / 2}`],
    explanation: "Divide numerator and denominator by their greatest common factor.",
    solvePattern: "pattern_spotting",
  };
}

function fractionAddTask(slot) {
  const a = 1 + (slot % 3);
  const b = 2 + (slot % 4);
  const denominator = 6;
  const answer = fraction(a + b, denominator);
  return {
    stem: `Find ${a}/${denominator} + ${b}/${denominator}.`,
    answer,
    distractors: [fraction(a + b, denominator * 2), fraction(a * b, denominator), fraction(a + b + 1, denominator)],
    explanation: "When denominators are the same, add the numerators and keep the denominator.",
    solvePattern: "substitute_and_solve",
  };
}

function fractionOperationTask(slot) {
  const numerator = 2 + (slot % 4);
  const denominator = 5 + (slot % 3);
  const quantity = denominator * (6 + (slot % 5));
  const answer = (quantity / denominator) * numerator;
  return {
    stem: `Find ${numerator}/${denominator} of ${quantity}.`,
    answer: formatNumber(answer),
    distractors: [formatNumber(quantity / denominator), formatNumber(quantity * denominator / numerator), formatNumber(quantity - answer)],
    explanation: "Divide the quantity by the denominator, then multiply by the numerator.",
    solvePattern: "substitute_and_solve",
  };
}

function ratioTask(slot) {
  const left = 2 + (slot % 4);
  const right = 3 + (slot % 5);
  const total = (left + right) * (10 + (slot % 6));
  const answer = (total / (left + right)) * left;
  return {
    stem: `A sum of ${total} cedis is shared in the ratio ${left}:${right}. What is the smaller share?`,
    answer: formatNumber(Math.min(answer, total - answer)),
    distractors: [formatNumber(total / left), formatNumber(total / right), formatNumber(total - Math.min(answer, total - answer))],
    explanation: "Add the ratio parts, find the value of one part, then multiply by the required number of parts.",
    solvePattern: "multi_step_reasoning",
  };
}

function linearRelationTask(slot) {
  const m = 2 + (slot % 4);
  const c = 1 + (slot % 5);
  const x = 3 + (slot % 6);
  const answer = m * x + c;
  return {
    stem: `For the relation y = ${m}x + ${c}, find y when x = ${x}.`,
    answer: formatNumber(answer),
    distractors: [formatNumber(m + x + c), formatNumber(m * (x + c)), formatNumber(m * x - c)],
    explanation: "Substitute the given x-value into the relation and simplify.",
    solvePattern: "substitute_and_solve",
  };
}

function algebraTask(slot) {
  const a = 2 + (slot % 5);
  const b = 3 + (slot % 4);
  const x = 4 + (slot % 6);
  const answer = a * x + b;
  return {
    stem: `Evaluate ${a}x + ${b} when x = ${x}.`,
    answer: formatNumber(answer),
    distractors: [formatNumber(a + x + b), formatNumber(a * (x + b)), formatNumber(a * x - b)],
    explanation: "Substitute the value of x, multiply first, then add the constant.",
    solvePattern: "substitute_and_solve",
  };
}

function equationTask(slot) {
  const a = 5 + (slot % 9);
  const x = 4 + (slot % 8);
  const b = x + a;
  return {
    stem: `Solve x + ${a} = ${b}.`,
    answer: `x = ${x}`,
    distractors: [`x = ${b + a}`, `x = ${a}`, `x = ${b - x}`],
    explanation: "Use the inverse operation: subtract the same number from both sides.",
    solvePattern: "substitute_and_solve",
  };
}

function inequalityTask(slot) {
  const a = 3 + (slot % 7);
  const x = 5 + (slot % 6);
  const b = x + a;
  return {
    stem: `Solve x + ${a} >= ${b}.`,
    answer: `x >= ${x}`,
    distractors: [`x <= ${x}`, `x >= ${b + a}`, `x = ${x}`],
    explanation: "Subtract the same number from both sides and keep the inequality direction.",
    solvePattern: "substitute_and_solve",
  };
}

function simultaneousTask(slot) {
  const x = 3 + (slot % 7);
  const y = 2 + (slot % 5);
  return {
    stem: `Solve the simultaneous equations x + y = ${x + y} and x - y = ${x - y}.`,
    answer: `x = ${x}, y = ${y}`,
    distractors: [`x = ${y}, y = ${x}`, `x = ${x + y}, y = ${x - y}`, `x = ${x - y}, y = ${y}`],
    explanation: "Add the equations to eliminate y, then substitute to find y.",
    solvePattern: "multi_step_reasoning",
  };
}

function angleTask(slot) {
  const angle = 35 + (slot % 8) * 5;
  const answer = 180 - angle;
  return {
    stem: `Two angles lie on a straight line. One angle is ${angle} degrees. Find the other angle.`,
    answer: `${answer} degrees`,
    distractors: [`${angle} degrees`, `${90 - (angle % 45)} degrees`, `${180 + angle} degrees`],
    explanation: "Angles on a straight line add up to 180 degrees.",
    solvePattern: "substitute_and_solve",
  };
}

function constructionTask(slot) {
  const prompts = [
    ["Which tool pair is needed for a standard angle bisector construction?", "Compass and straightedge"],
    ["What should remain unchanged when copying an angle with a compass?", "The compass radius for matching arcs"],
    ["Which line is drawn when constructing a perpendicular bisector?", "A line through the midpoint at 90 degrees"],
  ];
  const [stem, answer] = prompts[slot % prompts.length];
  return {
    stem,
    answer,
    distractors: ["Calculator and protractor only", "A random freehand line", "A line parallel to one ray only"],
    explanation: "Geometric construction depends on controlled arcs and straight lines, not measurement by guesswork.",
    solvePattern: "direct_retrieval",
  };
}

function perimeterTask(slot) {
  const radius = 7 + (slot % 4) * 7;
  const answer = 2 * 22 / 7 * radius;
  return {
    stem: `Using pi = 22/7, find the circumference of a circle with radius ${radius} cm.`,
    answer: `${formatNumber(answer)} cm`,
    distractors: [`${formatNumber(22 / 7 * radius)} cm`, `${formatNumber(radius * radius)} cm`, `${formatNumber(2 * radius)} cm`],
    explanation: "The circumference of a circle is 2 x pi x radius.",
    solvePattern: "substitute_and_solve",
  };
}

function triangleAreaTask(slot) {
  const base = 8 + (slot % 7);
  const height = 6 + (slot % 5);
  const answer = (base * height) / 2;
  return {
    stem: `Find the area of a triangle with base ${base} cm and height ${height} cm.`,
    answer: `${formatNumber(answer)} cm^2`,
    distractors: [`${formatNumber(base * height)} cm^2`, `${formatNumber(base + height)} cm^2`, `${formatNumber((base + height) / 2)} cm^2`],
    explanation: "The area of a triangle is one half times base times height.",
    solvePattern: "substitute_and_solve",
  };
}

function pythagorasTask(slot) {
  const triples = [
    [3, 4, 5],
    [5, 12, 13],
    [8, 15, 17],
    [7, 24, 25],
  ];
  const [a, b, c] = triples[slot % triples.length];
  return {
    stem: `A right-angled triangle has shorter sides ${a} cm and ${b} cm. Find the hypotenuse.`,
    answer: `${c} cm`,
    distractors: [`${a + b} cm`, `${Math.abs(b - a)} cm`, `${a * b} cm`],
    explanation: "Use Pythagoras' theorem: hypotenuse squared equals the sum of the squares of the other two sides.",
    solvePattern: "substitute_and_solve",
  };
}

function surfaceAreaTask(slot) {
  const l = 4 + (slot % 5);
  const w = 3 + (slot % 4);
  const h = 2 + (slot % 6);
  const answer = 2 * (l * w + l * h + w * h);
  return {
    stem: `Find the surface area of a cuboid with length ${l} cm, width ${w} cm, and height ${h} cm.`,
    answer: `${answer} cm^2`,
    distractors: [`${l * w * h} cm^2`, `${2 * (l + w + h)} cm^2`, `${l * w + l * h + w * h} cm^2`],
    explanation: "A cuboid has two of each rectangular face, so add the three face areas and double the sum.",
    solvePattern: "multi_step_reasoning",
  };
}

function vectorTask(slot) {
  const a = 1 + (slot % 5);
  const b = 2 + (slot % 4);
  const c = 3 + (slot % 6);
  const d = 1 + (slot % 3);
  return {
    stem: `Add the vectors (${a}, ${b}) and (${c}, ${d}).`,
    answer: `(${a + c}, ${b + d})`,
    distractors: [`(${a * c}, ${b * d})`, `(${a - c}, ${b - d})`, `(${a + d}, ${b + c})`],
    explanation: "Add corresponding vector components.",
    solvePattern: "substitute_and_solve",
  };
}

function transformationTask(slot, name) {
  const x = 2 + (slot % 5);
  const y = 3 + (slot % 4);
  if (name.includes("rotation")) {
    return {
      stem: `Rotate point (${x}, ${y}) 90 degrees anticlockwise about the origin.`,
      answer: `(-${y}, ${x})`,
      distractors: [`(${y}, -${x})`, `(-${x}, -${y})`, `(${x}, -${y})`],
      explanation: "A 90 degree anticlockwise rotation maps (x, y) to (-y, x).",
      solvePattern: "pattern_spotting",
    };
  }
  if (name.includes("translation")) {
    const dx = 2;
    const dy = -1;
    return {
      stem: `Translate point (${x}, ${y}) by vector (${dx}, ${dy}).`,
      answer: `(${x + dx}, ${y + dy})`,
      distractors: [`(${x - dx}, ${y - dy})`, `(${x + dy}, ${y + dx})`, `(${dx}, ${dy})`],
      explanation: "A translation adds the vector components to the original coordinates.",
      solvePattern: "substitute_and_solve",
    };
  }
  return {
    stem: `Reflect point (${x}, ${y}) in the x-axis.`,
    answer: `(${x}, -${y})`,
    distractors: [`(-${x}, ${y})`, `(-${x}, -${y})`, `(${y}, ${x})`],
    explanation: "Reflection in the x-axis keeps x the same and changes the sign of y.",
    solvePattern: "pattern_spotting",
  };
}

function enlargementTask(slot) {
  const x = 2 + (slot % 5);
  const y = 3 + (slot % 4);
  const scale = 2 + (slot % 3);
  return {
    stem: `Enlarge point (${x}, ${y}) by scale factor ${scale} from the origin.`,
    answer: `(${x * scale}, ${y * scale})`,
    distractors: [`(${x + scale}, ${y + scale})`, `(${x}, ${y})`, `(${x * scale}, ${y})`],
    explanation: "An enlargement from the origin multiplies each coordinate by the scale factor.",
    solvePattern: "substitute_and_solve",
  };
}

function centralTendencyTask(slot) {
  const data = [4 + (slot % 3), 6, 8, 8, 10 + (slot % 4)];
  const mean = data.reduce((sum, value) => sum + value, 0) / data.length;
  return {
    stem: `Find the mean of ${data.join(", ")}.`,
    answer: formatNumber(mean),
    distractors: [formatNumber(data[2]), formatNumber(Math.max(...data) - Math.min(...data)), formatNumber(data.reduce((s, v) => s + v, 0))],
    explanation: "The mean is the total of the data values divided by the number of values.",
    solvePattern: "substitute_and_solve",
  };
}

function dataTask(slot) {
  const boys = 12 + (slot % 8);
  const girls = 10 + (slot % 7);
  return {
    stem: `A frequency table shows ${boys} boys and ${girls} girls joined a club. What is the total frequency?`,
    answer: formatNumber(boys + girls),
    distractors: [formatNumber(Math.abs(boys - girls)), formatNumber(boys), formatNumber(girls)],
    explanation: "The total frequency is the sum of all category frequencies.",
    solvePattern: "graph_or_table_reading",
  };
}

function probabilityTask(slot, name) {
  const red = 3 + (slot % 5);
  const blue = 5 + (slot % 4);
  const total = red + blue;
  if (name.includes("dependent")) {
    return {
      stem: `A bag has ${red} red and ${blue} blue counters. A red counter is picked and not replaced. What is the probability that the next counter is red?`,
      answer: fraction(red - 1, total - 1),
      distractors: [fraction(red, total), fraction(red, total - 1), fraction(blue, total - 1)],
      explanation: "Without replacement, both the number of red counters and the total number of counters decrease by 1 after a red is picked.",
      solvePattern: "multi_step_reasoning",
    };
  }
  return {
    stem: `A bag has ${red} red and ${blue} blue counters. What is the probability of picking a red counter?`,
    answer: fraction(red, total),
    distractors: [fraction(blue, total), fraction(red, blue), fraction(total, red)],
    explanation: "Probability is favourable outcomes divided by total possible outcomes.",
    solvePattern: "direct_retrieval",
  };
}

function curriculumFitTask(topic, objective, family, grade, nearby) {
  const objectiveText = objective?.objective_text || topic.name;
  const answer = `A ${grade} task that asks learners to ${objectiveText.charAt(0).toLowerCase()}${objectiveText.slice(1)}`;
  return {
    stem: `Which option best matches the skill focus for ${topic.code}?`,
    answer,
    distractors: [
      `A task from a nearby but different focus: ${nearby}`,
      "A task that rewards guessing a keyword without doing the mathematics",
      "A task that ignores the stated grade-level skill progression",
    ],
    explanation: `The correct option mirrors the official skill focus for ${topic.code}.`,
    solvePattern: family.solvePattern,
  };
}

function misconceptionCode(record, fallbackIndex) {
  const text = `${record?.title ?? ""} ${record?.misconception_statement ?? ""}`.toLowerCase();
  if (text.includes("inverse") || text.includes("equation")) return "wrong_inverse_operation";
  if (text.includes("formula")) return "formula_selection_error";
  if (text.includes("label") || text.includes("term") || text.includes("confusion")) return "concept_label_confusion";
  if (text.includes("definition") || text.includes("process")) return "process_vs_definition_confusion";
  return ["surface_pattern_copy", "concept_label_confusion", "wrong_inverse_operation"][fallbackIndex % 3];
}

function makeQuestion(topic, meta, allTopics, topicObjectives, topicNodes, topicMisconceptions, family, localSlot, globalIndex) {
  const objective = pick(topicObjectives, localSlot, { objective_text: topic.name, cognitive_level: "understanding" });
  const skill = pick(topicNodes, localSlot, null);
  const misconception = pick(topicMisconceptions, localSlot, null);
  const task = buildTask(topic, objective, localSlot + globalIndex, family, allTopics);
  const optionTexts = uniqueOptions(task.answer, task.distractors);
  const options = attachOptionDiagnostics(
    rotateOptions(optionTexts, String(task.answer), localSlot),
    topic,
    topicMisconceptions,
    task,
  );
  const solutionBreakdown = buildSolutionBreakdown(topic, meta, skill, objective, task, options);
  const stem = mathifyText(task.stem);
  const explanationText = buildDetailedExplanation(
    topic,
    meta,
    skill,
    objective,
    task,
    solutionBreakdown,
  );
  const anchors = meta?.source_anchors ?? [];
  const cognitiveLevel = objective?.cognitive_level ?? (
    family.cognitiveDemand === "recall" ? "knowledge" : "application"
  );

  return {
    question: {
      stem,
      question_format: "mcq",
      topic_code: topic.code,
      family_code: `${slugCode(topic.code)}_${family.key}_V1`,
      explanation_text: explanationText,
      difficulty_level: makeDifficulty(topic, localSlot, family),
      estimated_time_seconds: family.key === "SPEED" ? 35 : family.key === "EXAM" ? 90 : 60,
      marks: family.key === "EXAM" || family.key === "TRANSFER" ? 2 : 1,
      source_type: "authored",
      source_ref: makeSourceRef(meta, topic),
      ...(skill?.canonical_title ? { primary_skill_title: skill.canonical_title, skill_titles: [skill.canonical_title] } : { skill_titles: [] }),
      cognitive_level: cognitiveLevel,
      options,
      seed_batch: SEED_BATCH,
      seed_slot: localSlot,
      seed_family_bucket: family.key,
      grade: meta?.grade ?? topicLevel(topic),
      strand_name: meta?.strand_name,
      substrand_name: meta?.substrand_name,
      source_anchors: anchors,
      objective_text: objective?.objective_text,
      skill_node_type: skill?.node_type,
      skill_title: skill?.canonical_title,
      intended_answer: task.answer,
      latex_stem: stem,
      latex_options: options.map((option) => ({
        label: option.option_label,
        text: option.option_text,
        is_correct: option.is_correct,
      })),
      latex_answer: mathifyText(String(task.answer)),
      solution_breakdown: solutionBreakdown,
      analytics_tags: {
        knowledge_role: family.knowledgeRole,
        cognitive_demand: family.cognitiveDemand,
        solve_pattern: task.solvePattern ?? family.solvePattern,
        pedagogic_function: family.pedagogicFunction,
      },
    },
    intelligence: {
      question_index: globalIndex,
      primary_knowledge_role: family.knowledgeRole,
      primary_cognitive_demand: family.cognitiveDemand,
      primary_solve_pattern: task.solvePattern ?? family.solvePattern,
      primary_pedagogic_function: family.pedagogicFunction,
      classification_confidence: 8800,
      knowledge_roles: [family.knowledgeRole],
      cognitive_demands: [family.cognitiveDemand],
      solve_patterns: [task.solvePattern ?? family.solvePattern],
      pedagogic_functions: [family.pedagogicFunction],
      content_grains: skill ? ["skill", "topic"] : ["topic"],
      misconception_exposures: [misconceptionCode(misconception, localSlot)],
    },
  };
}

function validateGenerated(topics, families, questions, intelligence) {
  const topicCodes = new Set(topics.map((topic) => topic.code));
  const familyCodes = new Set(families.map((family) => family.family_code));
  const counts = new Map();

  for (const question of questions) {
    if (!topicCodes.has(question.topic_code)) {
      throw new Error(`Question references unknown topic ${question.topic_code}`);
    }
    if (!familyCodes.has(question.family_code)) {
      throw new Error(`Question references unknown family ${question.family_code}`);
    }
    const correctCount = question.options.filter((option) => option.is_correct).length;
    if (correctCount !== 1) {
      throw new Error(`Question must have exactly one correct option: ${question.stem}`);
    }
    counts.set(question.topic_code, (counts.get(question.topic_code) ?? 0) + 1);
  }

  for (const topic of topics) {
    const count = counts.get(topic.code) ?? 0;
    if (count !== QUESTION_TARGET_PER_TOPIC) {
      throw new Error(`${topic.code} generated ${count} questions instead of ${QUESTION_TARGET_PER_TOPIC}`);
    }
  }

  if (questions.length !== intelligence.length) {
    throw new Error(`Expected one intelligence record per question; got ${questions.length} questions and ${intelligence.length} intelligence records`);
  }
}

function main() {
  const topics = readJson("curriculum/topics.json");
  const nodes = readJson("curriculum/nodes.json");
  const objectives = readJson("curriculum/objectives.json");
  const rawMisconceptions = readJson("curriculum/misconceptions.json");
  const manifest = readJson("manifest.json");
  const docSeed = JSON.parse(fs.readFileSync(DOC_SEED, "utf8"));

  const contentTopics = topics.filter((topic) => topic.node_type === "topic");
  const misconceptions = ensureTopicMisconceptions(contentTopics, rawMisconceptions);
  const topicMeta = new Map((docSeed.topics ?? []).map((topic) => [topic.topic_code, topic]));
  const nodesByTopic = byTopic(nodes);
  const objectivesByTopic = byTopic(objectives);
  const misconceptionsByTopic = byTopic(misconceptions);

  const families = [];
  const questions = [];
  const intelligence = [];

  for (const topic of contentTopics) {
    for (const family of FAMILY_BLUEPRINT) {
      families.push({
        family_code: `${slugCode(topic.code)}_${family.key}_V1`,
        family_name: `${topic.code} ${family.name}`,
        topic_code: topic.code,
        family_type: family.familyType,
        canonical_pattern: `${family.name} for ${topic.name}`,
        description: `Deterministic ${SEED_BATCH} family for ${topic.code}; supports ${family.knowledgeRole}, ${family.cognitiveDemand}, ${family.solvePattern}, and ${family.pedagogicFunction} analytics.`,
      });
    }

    let topicSlot = 0;
    for (const family of FAMILY_BLUEPRINT) {
      for (let familySlot = 0; familySlot < family.count; familySlot += 1) {
        const globalIndex = questions.length;
        const generated = makeQuestion(
          topic,
          topicMeta.get(topic.code),
          contentTopics,
          objectivesByTopic.get(topic.code) ?? [],
          nodesByTopic.get(topic.code) ?? [],
          misconceptionsByTopic.get(topic.code) ?? [],
          family,
          topicSlot,
          globalIndex,
        );
        questions.push(generated.question);
        intelligence.push(generated.intelligence);
        topicSlot += 1;
      }
    }
  }

  validateGenerated(contentTopics, families, questions, intelligence);

  manifest.question_count = questions.length;
  manifest.topic_count = topics.length;
  manifest.checksums = {
    ...(manifest.checksums ?? {}),
    seed_batch: SEED_BATCH,
    questions_per_content_topic: String(QUESTION_TARGET_PER_TOPIC),
    content_topic_count: String(contentTopics.length),
    question_family_count: String(families.length),
  };

  writeJson("questions/families.json", families);
  writeJson("questions/questions.json", questions);
  writeJson("questions/intelligence.json", intelligence);
  writeJson("curriculum/misconceptions.json", misconceptions);
  writeJson("manifest.json", manifest);

  console.log(`Generated ${questions.length} questions across ${contentTopics.length} topics.`);
  console.log(`Generated ${families.length} question families.`);
  console.log(`Ensured ${misconceptions.length} misconception records.`);
}

main();
