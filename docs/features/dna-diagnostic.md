# DNA Diagnostic

## Purpose

Identify academic weakness, cause, and recommended action through structured assessment.

## User role

Student, Parent, Super Admin

## Exact workflow

Launch a diagnostic, serve phase items, submit attempts, advance phases, complete the run, and synchronize resulting analytics into learner truth and reports.

## Inputs

Diagnostic launch parameters, item routing profiles, attempt payloads

## Outputs

Diagnostic result, skill results, recommendations, audience reports, longitudinal summaries

## Domain entities

Diagnostic run, phase item, skill result, cause card, recommendation, audience report

## Backend commands

`launch_diagnostic`, `list_diagnostic_phase_items`, `submit_diagnostic_attempt`, `advance_diagnostic_phase`, `complete_diagnostic_and_sync`

## UI screens

Diagnostic runner, result report, admin diagnostic inspector

## Edge cases

Constructed-response answers, incomplete phases, no matching item profile, cross-phase recomputation

## Acceptance criteria

The complete diagnostic loop runs without frontend-side scoring or recommendation logic.
