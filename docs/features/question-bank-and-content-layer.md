# Question Bank and Content Layer

## Purpose

Provide the fuel for diagnostics, sessions, drills, and remediation through structured questions and content intelligence.

## User role

Student, Super Admin

## Exact workflow

Install or generate content, classify questions, review families, detect duplicates, and retrieve aligned material for diagnostics or study.

## Inputs

Content packs, generation requests, question reviews, retrieval requests

## Outputs

Question intelligence profiles, related questions, resource intelligence, remediation candidates

## Domain entities

Question, question family, lineage edge, content artifact, resource intelligence profile

## Backend commands

`install_pack`, `create_question_generation_request`, `process_question_generation_request`, `get_question_intelligence`, `orchestrate_resource_plan`

## UI screens

Question intelligence, content foundry, related-question inspectors

## Edge cases

Near duplicates, under-filled families, generated-only inventory, low-confidence classifications

## Acceptance criteria

The backend can supply explainable question and resource selection without manual frontend stitching.
