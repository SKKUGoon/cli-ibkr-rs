# OAuth Session Instructions

- This module owns the IBKR OAuth 1.0a Live Session Token lifecycle.
- Do not add Client Portal Gateway authentication or alternate auth modes.
- Keep the LST request flow explicit and auditable.
- Do not continue after OAuth, key, cache, or validation errors.
