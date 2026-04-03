# Super Admin Foundation

## Purpose

Give one role control over curriculum, content, question quality, runtime protection, and operational oversight.

## User role

Super Admin

## Exact workflow

Enter admin shell, inspect academic structures, manage users and content operations, then run recovery actions when needed.

## Inputs

Curriculum edits, content uploads, review actions, backup paths

## Outputs

Published academic state, user changes, recovery artifacts

## Domain entities

Account, curriculum version, source upload, foundry job, recovery snapshot

## Backend commands

Curriculum save and publish commands, content foundry commands, question review commands, recovery commands

## UI screens

Admin dashboard, curriculum manager, content foundry, question intelligence, recovery workspace

## Edge cases

Missing reference docs, inconsistent source material, invalid backup destinations

## Acceptance criteria

Admin-facing operations remain command-driven and can protect the runtime state without direct filesystem manipulation from the UI.
