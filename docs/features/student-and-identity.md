# Student and Identity

## Purpose

Represent the learner as an account plus a continually updated academic truth model.

## User role

Student, Parent

## Exact workflow

Authenticate, load learner truth, update the learner through attempts and session completion, then expose summaries to student and parent surfaces.

## Inputs

Identity records, attempt data, session outcomes, role links

## Outputs

Learner truth snapshots, dashboard data, entitlement-aware visibility

## Domain entities

Account, learner truth, topic state, confidence profile, pressure profile

## Backend commands

`login_with_pin`, `get_learner_truth`, `get_student_dashboard`, learner and premium reporting surfaces

## UI screens

Login, student dashboard, parent child-detail views

## Edge cases

Missing student linkage, unsupported entitlement state, stale learner snapshots

## Acceptance criteria

The learner model stays backend-owned and is readable consistently across student and parent experiences.
