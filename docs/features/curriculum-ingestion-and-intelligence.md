# Curriculum Ingestion and Intelligence

## Purpose

Turn raw academic source material into a canonical curriculum graph with provenance and publishing control.

## User role

Super Admin, Student

## Exact workflow

Register a curriculum source, add parse candidates, finalize the source, review the result, and publish a version for student use.

## Inputs

Source uploads, parse candidates, node bundles, citations, exemplars, review comments

## Outputs

Published curriculum versions, subject trees, remediation and prerequisite maps

## Domain entities

Curriculum family, version, subject, topic, node intelligence, source upload, publish job

## Backend commands

`register_curriculum_source`, `add_curriculum_parse_candidate`, `finalize_curriculum_source`, curriculum save and publish commands

## UI screens

Curriculum manager, ingestion workspace, admin node detail

## Edge cases

Conflicting source structures, unpublished versions, missing citations, regeneration backlog

## Acceptance criteria

Students and admin read from the same published academic truth, not from ad hoc UI state.
