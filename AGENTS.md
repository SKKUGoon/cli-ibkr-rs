# Repository Instructions

- This workspace implements an OAuth-only IBKR CLI worker.
- Do not add Client Portal Gateway, HTTP job API, Redis queue, Kafka, WebSocket, watchlist, scanner, or contract lookup code.
- Redis usage is allowed only for the centralized OAuth Live Session Token cache.
- Keep command output machine-safe: JSON goes to stdout, diagnostics and logs go to stderr.
- Prefer propagating real upstream errors over fallback behavior.
