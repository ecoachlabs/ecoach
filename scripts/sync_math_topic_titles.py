from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import openpyxl


WORKBOOK_PATH = Path(r"C:\Users\surfaceSudio\OneDrive\ecoach\outputs\mathematics_topic_mapping_corrected.xlsx")
DOCS_DIR = Path(r"C:\Users\surfaceSudio\OneDrive\ecoach\docs\curriculum\math_ghana_ccp_b7_b9")
PACK_DIR = Path(r"C:\Users\surfaceSudio\OneDrive\ecoach\packs\math-ghana-ccp-b7b9-foundation")

EXTRA_TOPIC_MAP = {
    "B7.1.1.2": "Comparing and Ordering Whole Numbers",
}

EXTRA_OFFICIAL_LABEL_MAP = {
    "B7.1.1.2": 'Compare and order whole numbers more than1,000,000,000 and represent the comparison using ">, <, or="',
}


def load_topic_maps() -> tuple[dict[str, str], dict[str, str]]:
    workbook = openpyxl.load_workbook(WORKBOOK_PATH, read_only=True)
    sheet = workbook["All_Standards"]
    topic_map: dict[str, str] = {}
    official_label_map: dict[str, str] = {}

    for row in sheet.iter_rows(min_row=2, values_only=True):
        code = row[3]
        official_label = row[5]
        title = row[6]
        if code and title:
            topic_map[str(code)] = str(title)
        if code and official_label:
            official_label_map[str(code)] = str(official_label).rstrip(".")

    topic_map.update(EXTRA_TOPIC_MAP)
    official_label_map.update(EXTRA_OFFICIAL_LABEL_MAP)
    return topic_map, official_label_map


def load_json(path: Path) -> Any:
    return json.load(path.open(encoding="utf-8"))


def save_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def replace_prefix_with_grade_suffix(text: str, new_title: str) -> str:
    marker = " (Grade "
    idx = text.find(marker)
    if idx != -1:
        return f"{new_title}{text[idx:]}"
    return new_title


def replace_strings(value: Any, replacements: dict[str, str]) -> Any:
    if isinstance(value, str):
        for old, new in replacements.items():
            value = value.replace(old, new)
        return value
    if isinstance(value, list):
        return [replace_strings(item, replacements) for item in value]
    if isinstance(value, dict):
        return {key: replace_strings(item, replacements) for key, item in value.items()}
    return value


def sync_seed_package(topic_map: dict[str, str]) -> int:
    path = DOCS_DIR / "mathematics_ccp_seed_package.json"
    data = load_json(path)
    updated = 0
    for topic in data.get("topics", []):
        code = topic.get("topic_code")
        if code in topic_map:
            topic["topic_name"] = topic_map[code]
            updated += 1
    save_json(path, data)
    return updated


def sync_semantic_map(topic_map: dict[str, str]) -> int:
    path = DOCS_DIR / "mathematics_ccp_education_semantic_map.json"
    data = load_json(path)
    updated = 0
    for topic in data.get("topics", []):
        code = topic.get("topic_code")
        if code in topic_map:
            topic["topic_name"] = topic_map[code]
            updated += 1
    save_json(path, data)
    return updated


def sync_interactive_blueprint(topic_map: dict[str, str]) -> int:
    path = DOCS_DIR / "mathematics_ccp_interactive_blueprint.json"
    data = load_json(path)
    updated = 0
    for topic in data.get("data_index", {}).get("topic_nodes", []):
        code = topic.get("topic_code")
        if code in topic_map:
            topic["topic_name"] = topic_map[code]
            updated += 1
    save_json(path, data)
    return updated


def sync_pack_topics(topic_map: dict[str, str], official_label_map: dict[str, str]) -> int:
    path = PACK_DIR / "curriculum" / "topics.json"
    data = load_json(path)
    updated = 0
    for topic in data:
        code = topic.get("code")
        if code in topic_map and topic.get("node_type") == "topic":
            topic["name"] = topic_map[code]
            topic["description"] = official_label_map.get(code, topic.get("description"))
            updated += 1
    save_json(path, data)
    return updated


def sync_explanations(topic_map: dict[str, str]) -> int:
    path = PACK_DIR / "content" / "explanations.json"
    data = load_json(path)
    updated = 0
    for entry in data:
        code = entry.get("topic_code")
        if code in topic_map:
            title = topic_map[code]
            entry["short_text"] = title
            entry["full_text"] = replace_prefix_with_grade_suffix(entry.get("full_text", ""), title)
            entry["simple_text"] = f"Learn {title} with examples and practice."
            entry["technical_text"] = title
            updated += 1
    save_json(path, data)
    return updated


def sync_formulas(topic_map: dict[str, str]) -> int:
    path = PACK_DIR / "content" / "formulas.json"
    data = load_json(path)
    updated = 0
    for entry in data:
        code = entry.get("topic_code")
        if code in topic_map:
            title = topic_map[code]
            entry["short_text"] = f"Formula and symbolic patterns for {title}."
            entry["full_text"] = f"Compile and verify formulas for {title} before assessment practice."
            entry["aliases"] = [title]
            updated += 1
    save_json(path, data)
    return updated


def sync_worked_examples(topic_map: dict[str, str]) -> int:
    path = PACK_DIR / "content" / "worked_examples.json"
    data = load_json(path)
    updated = 0
    for entry in data:
        code = entry.get("topic_code")
        if code in topic_map:
            title = topic_map[code]
            entry["short_text"] = f"Step-by-step solving pattern for {title}"
            full_text = entry.get("full_text", "")
            suffix = " using the official indicator intent"
            if suffix in full_text:
                entry["full_text"] = f"Model a worked solution for {title}{full_text[full_text.index(suffix):]}"
            else:
                entry["full_text"] = f"Model a worked solution for {title}."
            updated += 1
    save_json(path, data)
    return updated


def sync_question_families(topic_map: dict[str, str]) -> int:
    path = PACK_DIR / "questions" / "families.json"
    data = load_json(path)
    updated = 0
    for family in data:
        code = family.get("topic_code")
        if code in topic_map:
            family["canonical_pattern"] = f"{family.get('family_name', code)} for {topic_map[code]}"
            updated += 1
    save_json(path, data)
    return updated


def sync_questions(topic_map: dict[str, str], official_label_map: dict[str, str]) -> int:
    path = PACK_DIR / "questions" / "questions.json"
    data = load_json(path)
    updated = 0

    for index, question in enumerate(data):
        code = question.get("topic_code")
        if code not in topic_map:
            continue

        title = topic_map[code]
        official_label = official_label_map.get(code)
        replacements = {official_label: title} if official_label else {}
        if replacements:
            data[index] = replace_strings(question, replacements)
            question = data[index]

        breakdown = question.get("solution_breakdown")
        if isinstance(breakdown, dict):
            curriculum_link = breakdown.get("curriculum_link")
            if isinstance(curriculum_link, dict):
                curriculum_link["topic_name"] = title

        updated += 1

    save_json(path, data)
    return updated


def verify_pack_files_have_no_old_labels(paths: list[Path], official_label_map: dict[str, str]) -> list[tuple[Path, str]]:
    failures: list[tuple[Path, str]] = []
    labels = list(official_label_map.values())
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for label in labels:
            if label in text:
                failures.append((path, label))
    return failures


def main() -> None:
    topic_map, official_label_map = load_topic_maps()
    counts = {
        "seed_package": sync_seed_package(topic_map),
        "semantic_map": sync_semantic_map(topic_map),
        "interactive_blueprint": sync_interactive_blueprint(topic_map),
        "pack_topics": sync_pack_topics(topic_map, official_label_map),
        "explanations": sync_explanations(topic_map),
        "formulas": sync_formulas(topic_map),
        "worked_examples": sync_worked_examples(topic_map),
        "question_families": sync_question_families(topic_map),
        "questions": sync_questions(topic_map, official_label_map),
    }

    checked_paths = [
        PACK_DIR / "content" / "explanations.json",
        PACK_DIR / "content" / "formulas.json",
        PACK_DIR / "content" / "worked_examples.json",
        PACK_DIR / "questions" / "families.json",
        PACK_DIR / "questions" / "questions.json",
    ]
    failures = verify_pack_files_have_no_old_labels(checked_paths, official_label_map)

    print("Updated counts:", counts)
    print("Verification failures:", len(failures))
    for path, label in failures:
        print(f"{path} -> {label}")


if __name__ == "__main__":
    main()
