# Session Runtime

## Purpose

Execute real learning sessions, collect evidence, and feed downstream models.

## User role

Student

## Exact workflow

Start a practice or coach session, serve prompts, record attempts and presence, then complete the session through the post-session pipeline.

## Inputs

Session start inputs, attempts, focus mode settings, presence events

## Outputs

Session summaries, attempt results, evidence records, completion cascades

## Domain entities

Session, session item, attempt, focus config, presence event

## Backend commands

`start_practice_session`, `start_coach_mission_session`, `submit_attempt`, `complete_session_with_pipeline`, `enable_focus_mode`

## UI screens

Session runner, pause/resume controls, completion summary

## Edge cases

Manual stop, pause and resume, offline interruption, partial attempt submission

## Acceptance criteria

Session completion updates the backend truth and does not end as a frontend-only interaction.
