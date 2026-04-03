# Study Plan and Journey

## Purpose

Convert evidence, time, and deadlines into a next-best learning route.

## User role

Student, Parent

## Exact workflow

Read learner truth, generate or refresh a route, adapt it using new evidence, and expose daily missions and schedule-aware decisions.

## Inputs

Learner truth, academic events, availability, goals, readiness

## Outputs

Journey route, daily mission, goal recommendations, schedule recommendations

## Domain entities

Journey route, mission, milestone, academic event, availability profile

## Backend commands

`build_or_refresh_journey_route`, `get_active_journey_route`, `generate_today_mission`, `adapt_journey_route`, `get_goal_recommendation`

## UI screens

Student dashboard, study path, parent summary views

## Edge cases

Low available time, deadline compression, contradictory goals, missed sessions

## Acceptance criteria

Next actions remain explainable and are recomputed from backend evidence instead of hard-coded UI flows.
