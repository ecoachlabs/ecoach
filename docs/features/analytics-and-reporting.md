# Analytics and Reporting

## Purpose

Turn backend evidence into readable student, parent, and admin insight.

## User role

Student, Parent, Super Admin

## Exact workflow

Aggregate learner and diagnostic state, build audience-specific summaries, and expose risk, readiness, and intervention signals.

## Inputs

Learner truth, diagnostic results, momentum, calendar state, premium strategy data

## Outputs

Dashboards, alerts, readiness summaries, audience reports, oversight views

## Domain entities

Readiness report, parent alert, strategy summary, oversight snapshot

## Backend commands

Reporting, readiness, diagnostic audience, premium risk, and parent alert command surfaces

## UI screens

Student reports, parent overview, admin oversight, strategy summary screens

## Edge cases

Sparse data, conflicting signals, stale summaries, premium-only detail access

## Acceptance criteria

Reports read directly from backend-computed DTOs and remain role-aware.
