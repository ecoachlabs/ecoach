# Core Platform Foundation

## Purpose

Boot the local desktop app, authenticate the current user, and make the runtime safe to operate.

## User role

Student, Parent, Super Admin

## Exact workflow

Open app, resolve runtime state, authenticate with PIN, load the correct shell, and keep recovery tools available.

## Inputs

Account creation data, PIN, entitlement tier, runtime paths

## Outputs

Account DTOs, authenticated session context, recovery status

## Domain entities

Account, role, entitlement, runtime database, rebuild workspace status

## Backend commands

`list_accounts`, `create_account`, `login_with_pin`, `inspect_rebuild_workspace`

## UI screens

Login / PIN, shell loader, admin recovery entry

## Edge cases

Invalid PIN, missing runtime database, missing rebuild docs, poisoned runtime state

## Acceptance criteria

The app boots, users can authenticate, and recovery inspection is reachable without frontend-side business logic.
