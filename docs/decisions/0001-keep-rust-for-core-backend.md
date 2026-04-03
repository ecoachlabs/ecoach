# ADR 0001: Keep Rust for the Core Desktop Backend

## Status

Accepted

## Context

Idea35 explicitly re-opened the stack choice after the crash and asked whether the rebuild should switch from Rust to Go.

## Decision

Keep Rust for the core desktop backend and continue using Tauri as the desktop boundary.

## Why

- The current app architecture is already Tauri-native.
- The existing backend crates, migrations, and command surface are in Rust.
- A Go rewrite would add a framework/runtime migration at the same time as the product rebuild.
- The recovery goal is to stabilize contracts, not introduce a second major platform change.

## Consequence

Go can still be introduced later for isolated utilities, but not as the main runtime path for this rebuild.
