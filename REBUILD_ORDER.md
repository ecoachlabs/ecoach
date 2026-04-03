# Rebuild Order

This rebuild order follows `ideas/idea35.txt`, but it is grounded in the backend that already exists.

## Slice 0: Recovery Safety

1. Keep the five root rebuild docs current.
2. Keep truth packs under `docs/features/`.
3. Export runtime backups and recovery snapshots after major slices.
4. Keep the command boundary stable while frontend catches up.

## Slice 1: Core Platform Foundation

Scope:
- identity and PIN access
- role handling
- app state bootstrap
- recovery tooling entry points

Acceptance:
- accounts can be created and used
- the runtime boots cleanly
- recovery commands are callable

## Slice 2: Super Admin Foundation

Scope:
- user oversight
- curriculum governance
- content and question administration
- recovery workspace inspection

Acceptance:
- admin can inspect and manage academic structure
- admin can see recovery status and export protections

## Slice 3: Curriculum and Content Spine

Scope:
- curriculum graph
- source ingestion
- publishing and resource intelligence

Acceptance:
- curriculum can be created, reviewed, and published
- topic resources can be resolved from backend contracts

## Slice 4: Question and Diagnostic Spine

Scope:
- question intelligence
- diagnostic launch and progression
- reporting of causes and recommendations

Acceptance:
- diagnostics run end to end
- question selection and lineage are traceable

## Slice 5: Student Truth and Journey

Scope:
- learner truth
- journey generation
- goal and schedule orchestration

Acceptance:
- learner state drives next action and route decisions

## Slice 6: Session Runtime and Memory Loop

Scope:
- session execution
- evidence recording
- memory review and rechecks

Acceptance:
- session completion updates downstream state
- due reviews surface correctly

## Slice 7: Reporting and Parent Surfaces

Scope:
- readiness summaries
- audience reports
- parent alerts and strategy guidance

Acceptance:
- reports are generated from backend truth, not frontend derivation

## Slice 8: Library, Glossary, and Drill Surfaces

Scope:
- saved learning assets
- glossary teaching and tests
- games and traps

Acceptance:
- supporting study surfaces are command-reachable and stateful

## Slice 9: Frontend Reconstruction

Scope:
- thin shell first
- one vertical slice at a time
- no UI-only logic forks

Acceptance:
- each screen is backed by an existing command contract and a truth pack

## Definition of Done

A slice is complete only when it has:
- backend logic
- storage and migrations if needed
- command exposure
- focused tests
- a truth pack or recovery note
- a screen contract in the inventory
