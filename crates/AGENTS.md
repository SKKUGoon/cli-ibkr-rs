# Crates Instructions

- Keep reusable IBKR protocol logic in `core`.
- Keep CLI parsing, process exit behavior, and local OpenSSL execution in `worker`.
- Split files before they grow beyond roughly 150 lines.
- Use `module.rs` only for leaf modules.
- When a module has child files, use `module/mod.rs`.
- Put implementation slices beside `mod.rs`, for example `request.rs`, `response.rs`, or `validation.rs`.
- Put module-local tests in `module/tests.rs` when tests are large enough to make the main file noisy.
