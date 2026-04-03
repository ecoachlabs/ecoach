# Memory, Decay, and Retention

## Purpose

Keep learning durable by detecting forgetting, scheduling rechecks, and surfacing interventions.

## User role

Student

## Exact workflow

Record retrieval evidence, build review queues, process decay scans, complete rechecks, and update topic knowledge state.

## Inputs

Retrieval attempts, elapsed time, interference, interventions

## Outputs

Review queue items, decay batches, return loops, topic memory summaries

## Domain entities

Memory state, recheck item, intervention, interference edge, knowledge map node

## Backend commands

`get_review_queue`, `build_memory_review_queue`, `record_retrieval_attempt`, `process_decay_batch`, `complete_recheck`

## UI screens

Memory review, due-now list, knowledge map

## Edge cases

Interference spikes, overdue items, repeated failure, forced recomputation

## Acceptance criteria

Retention timing and memory status are computed in the backend and feed the coach and reporting layers.
