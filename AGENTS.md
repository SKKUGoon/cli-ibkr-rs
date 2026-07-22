# AGENTS.md

Root guide for AI agents working in this repository.

## Stack and Architecture

This project is a Rust 2021 workspace implementing an OAuth-only IBKR CLI for
Airflow and manual operator tasks.

- Reusable IBKR protocol, OAuth, REST, and domain-model code lives in
  `crates/core`.
- CLI parsing, process exit behavior, operator interaction, output handling,
  local OpenSSL execution, and optional database persistence live in
  `crates/worker`.
- Integration and CLI behavior tests live in `tests` and crate-local `tests`
  modules.
- Use `cargo fmt`, `cargo clippy --workspace --all-targets --all-features`, and
  `cargo test --workspace` to verify changes.

Before editing a scoped area, read the closest `AGENTS.md`. A more specific
guide supplements this root guide and wins when its instructions conflict.

## Product Boundaries

- Do not add Client Portal Gateway, HTTP job API, Redis queue, Kafka,
  WebSocket, watchlist, scanner, or broad contract-lookup code.
- Stock symbol-to-conid resolution is supported for order and market-data
  workflows.
- Redis is allowed only for the centralized OAuth Live Session Token cache.
- Keep machine commands safe for orchestration: JSON goes to stdout;
  diagnostics and logs go to stderr.
- Interactive operator utilities must be clearly separated in CLI help and
  must not be used as hidden fallbacks for machine commands.
- Prefer propagating authentic upstream, configuration, and validation errors
  over fallback behavior.

## Coding Convention

### 1. A function must be understandable from its name and signature alone

A reader at the call site must be able to tell what a function does without
opening its body or following nested definitions.

- Name functions with an action and target. Avoid vague names such as
  `process`, `handle`, `run_task`, or `do_work`; prefer names such as
  `resolve_stock_conid`, `submit_vwap_order`, or
  `write_command_json_output`.
- Name variables for their contents. Avoid `res`, `tmp`, `data`, or `value`
  when a domain name such as `account_summary_response`, `selected_side`, or
  `validated_order_request` is available.
- Name booleans as predicates: `is_market_open`, `has_selected_account`, or
  `should_include_outside_rth`.
- Surface side effects in names. Network calls, database writes, file writes,
  cache mutations, and order submission must not hide behind neutral names.
- Comments explain constraints, tradeoffs, protocol requirements, and reasons.
  Do not narrate code that already reads clearly.

### 2. Prefer typed, explicit Rust interfaces

Keep boundaries clear between CLI orchestration, operator interaction,
database/API calls, and pure domain logic.

- Use structs, enums, and small domain types for non-trivial request shapes and
  state transitions. Avoid passing loosely structured `serde_json::Value`
  through multiple internal layers when a stable shape is known.
- Preserve raw IBKR JSON at the CLI output boundary unless a command explicitly
  documents a transformed schema.
- Keep CLI command functions thin. Put reusable protocol and validation logic
  in `core`; keep prompts, terminal behavior, and process concerns in `worker`.
- Return named structs instead of tuples whose positions must be remembered.
- Accept borrowed values (`&str`, slices, references) when ownership is not
  required. Take ownership when the callee stores, transforms, or consumes the
  value.
- Use constructors and validation methods when invalid domain states can be
  prevented. Do not expose public fields solely to avoid writing a clear API.
- Use enums for closed choices such as order side or strategy mode. Preserve a
  forward-compatible raw representation only where IBKR genuinely has an open
  set of values.
- Keep async functions at I/O boundaries. Pure parsing, validation, and payload
  construction should remain synchronous and directly unit-testable.

### 3. No silent fallback to old code

When replacing an implementation, the new code replaces the old one. Do not
quietly catch an error and delegate to a legacy path.

- Replace fully and remove the superseded path in the same change.
- If fallback behavior is genuinely required, gate it behind a named option,
  emit a diagnostic when it activates, and document it.
- Propagate failures with `?` and specific error variants. Do not convert a
  failed network call, cache operation, parse, or validation into an empty
  success value.
- Do not add hidden authentication paths, retries, or recovery branches. IBKR
  session preflight operations must be explicit and observable.

### 4. Keep modules and command boundaries focused

- Each machine command performs one documented IBKR operation and exits.
- Interactive utilities may gather operator input and then submit one
  documented operation; keep their prompt code separate from reusable payload
  construction and REST code.
- Split files before they grow beyond roughly 150 lines. Prefer objective-based
  modules and place large tests in a sibling `tests.rs`.
- Do not duplicate endpoint strings or wire-format field names across command
  modules. Centralize them in endpoint and model modules.

### 5. Verification is part of the change

- Add unit tests for payload construction, parsing, validation, and CLI
  argument behavior.
- CLI-facing changes must verify stdout/stderr separation and exit behavior
  where applicable.
- Tests must not call the live IBKR API.
- Run formatting, Clippy with warnings denied, and the workspace test suite
  before handing off changes.
