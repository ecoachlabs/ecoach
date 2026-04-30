from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path
from typing import Any

import openpyxl


WORKBOOK_PATH = Path(r"C:\Users\surfaceSudio\OneDrive\ecoach\outputs\mathematics_topic_mapping_corrected.xlsx")
PACK_DIR = Path(r"C:\Users\surfaceSudio\OneDrive\ecoach\packs\math-ghana-ccp-b7b9-foundation")
DEFAULT_DB_PATH = Path(r"C:\Users\surfaceSudio\AppData\Roaming\com.ecoach.app\ecoach.db")
PACK_ID = "math-ghana-ccp-b7b9-foundation-v1"

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


def replace_text(text: str | None, replacements: dict[str, str]) -> str | None:
    if text is None:
        return None
    for old, new in replacements.items():
        text = text.replace(old, new)
    return text


def update_topics(conn: sqlite3.Connection, pack_topics: list[dict[str, Any]]) -> int:
    updated = 0
    for topic in pack_topics:
        cursor = conn.execute(
            """
            UPDATE topics
               SET name = ?1,
                   description = ?2,
                   updated_at = datetime('now')
             WHERE code = ?3
            """,
            (topic.get("name"), topic.get("description"), topic.get("code")),
        )
        updated += cursor.rowcount
    return updated


def update_curriculum_nodes(conn: sqlite3.Connection) -> int:
    cursor = conn.execute(
        """
        UPDATE curriculum_nodes
           SET canonical_title = (SELECT name FROM topics WHERE id = curriculum_nodes.legacy_topic_id),
               public_title = (SELECT name FROM topics WHERE id = curriculum_nodes.legacy_topic_id),
               official_text = (SELECT description FROM topics WHERE id = curriculum_nodes.legacy_topic_id),
               public_summary = (SELECT description FROM topics WHERE id = curriculum_nodes.legacy_topic_id),
               updated_at = datetime('now')
         WHERE legacy_topic_id IS NOT NULL
        """
    )
    return cursor.rowcount


def update_knowledge_entries(conn: sqlite3.Connection, entries: list[dict[str, Any]]) -> int:
    updated = 0
    for entry in entries:
        cursor = conn.execute(
            """
            UPDATE knowledge_entries
               SET title = ?1,
                   canonical_name = ?2,
                   short_text = ?3,
                   full_text = ?4,
                   simple_text = ?5,
                   technical_text = ?6,
                   importance_score = ?7,
                   updated_at = datetime('now')
             WHERE entry_type = ?8
               AND topic_id = (SELECT id FROM topics WHERE code = ?9 LIMIT 1)
            """,
            (
                entry.get("title"),
                entry.get("canonical_name"),
                entry.get("short_text"),
                entry.get("full_text"),
                entry.get("simple_text"),
                entry.get("technical_text"),
                entry.get("importance_score"),
                entry.get("entry_type"),
                entry.get("topic_code"),
            ),
        )
        updated += cursor.rowcount
    return updated


def update_question_families(conn: sqlite3.Connection, families: list[dict[str, Any]]) -> int:
    updated = 0
    for family in families:
        cursor = conn.execute(
            """
            UPDATE question_families
               SET family_name = ?1,
                   canonical_pattern = ?2,
                   description = ?3,
                   updated_at = datetime('now')
             WHERE family_code = ?4
            """,
            (
                family.get("family_name"),
                family.get("canonical_pattern"),
                family.get("description"),
                family.get("family_code"),
            ),
        )
        updated += cursor.rowcount
    return updated


def build_question_row_index(
    conn: sqlite3.Connection,
) -> dict[tuple[str, str | None, str | None, str], list[tuple[int, str | None]]]:
    rows = conn.execute(
        """
        SELECT q.id, t.code, f.family_code, q.source_ref, q.stem, q.intelligence_snapshot
          FROM questions q
          JOIN topics t ON t.id = q.topic_id
          LEFT JOIN question_families f ON f.id = q.family_id
         WHERE q.pack_id = ?
        """,
        (PACK_ID,),
    ).fetchall()
    index: dict[tuple[str, str | None, str | None, str], list[tuple[int, str | None]]] = {}
    for question_id, code, family_code, source_ref, stem, intelligence_snapshot in rows:
        key = (code, family_code, source_ref, stem)
        index.setdefault(key, []).append((question_id, intelligence_snapshot))
    return index


def update_questions(
    conn: sqlite3.Connection,
    questions: list[dict[str, Any]],
    official_label_map: dict[str, str],
    topic_map: dict[str, str],
) -> tuple[int, int]:
    row_index = build_question_row_index(conn)
    explanation_updates = 0
    snapshot_updates = 0

    for question in questions:
        key = (
            question.get("topic_code"),
            question.get("family_code"),
            question.get("source_ref"),
            question.get("stem"),
        )
        matches = row_index.get(key)
        if not matches:
            continue

        question_id, intelligence_snapshot = matches.pop(0)
        code = str(question.get("topic_code"))
        old_label = official_label_map.get(code)
        title = topic_map.get(code)
        replacements = {old_label: title} if old_label and title else {}
        snapshot_value = replace_text(intelligence_snapshot, replacements) if replacements else intelligence_snapshot

        cursor = conn.execute(
            """
            UPDATE questions
               SET explanation_text = ?1,
                   intelligence_snapshot = ?2,
                   updated_at = datetime('now')
             WHERE id = ?3
            """,
            (question.get("explanation_text"), snapshot_value, question_id),
        )
        explanation_updates += cursor.rowcount
        if intelligence_snapshot != snapshot_value:
            snapshot_updates += cursor.rowcount

    return explanation_updates, snapshot_updates


def find_remaining_old_labels(
    conn: sqlite3.Connection,
    official_label_map: dict[str, str],
) -> list[tuple[str, str, int]]:
    targets = [
        ("topics", "name"),
        ("curriculum_nodes", "canonical_title"),
        ("curriculum_nodes", "public_title"),
        ("knowledge_entries", "short_text"),
        ("knowledge_entries", "full_text"),
        ("knowledge_entries", "simple_text"),
        ("knowledge_entries", "technical_text"),
        ("question_families", "canonical_pattern"),
        ("questions", "explanation_text"),
        ("questions", "intelligence_snapshot"),
    ]

    failures: list[tuple[str, str, int]] = []
    for label in official_label_map.values():
        for table, column in targets:
            count = conn.execute(
                f"SELECT COUNT(*) FROM {table} WHERE {column} IS NOT NULL AND instr({column}, ?) > 0",
                (label,),
            ).fetchone()[0]
            if count:
                failures.append((f"{table}.{column}", label, count))
    return failures


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Sync corrected math topic titles into the runtime database.")
    parser.add_argument("--db", type=Path, default=DEFAULT_DB_PATH)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    topic_map, official_label_map = load_topic_maps()
    pack_topics = load_json(PACK_DIR / "curriculum" / "topics.json")
    explanations = load_json(PACK_DIR / "content" / "explanations.json")
    formulas = load_json(PACK_DIR / "content" / "formulas.json")
    worked_examples = load_json(PACK_DIR / "content" / "worked_examples.json")
    families = load_json(PACK_DIR / "questions" / "families.json")
    questions = load_json(PACK_DIR / "questions" / "questions.json")

    conn = sqlite3.connect(args.db)
    conn.execute("PRAGMA foreign_keys = ON")

    try:
        with conn:
            counts = {
                "topics": update_topics(conn, pack_topics),
                "curriculum_nodes": update_curriculum_nodes(conn),
                "knowledge_entries": update_knowledge_entries(
                    conn,
                    explanations + formulas + worked_examples,
                ),
                "question_families": update_question_families(conn, families),
            }
            question_updates, snapshot_updates = update_questions(
                conn,
                questions,
                official_label_map,
                topic_map,
            )
            counts["questions"] = question_updates
            counts["intelligence_snapshots"] = snapshot_updates

        failures = find_remaining_old_labels(conn, official_label_map)
        print("Updated counts:", counts)
        print("Verification failures:", len(failures))
        for location, label, count in failures[:50]:
            safe_label = label.encode("ascii", "backslashreplace").decode("ascii")
            print(f"{location} -> {count} occurrences of {safe_label}")
        if failures:
            raise SystemExit(1)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
