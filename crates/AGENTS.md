# Crates Instructions

- Keep reusable IBKR protocol logic in `core`.
- Keep CLI parsing, process exit behavior, and local OpenSSL execution in `worker`.
- Split files before they grow beyond roughly 150 lines.
