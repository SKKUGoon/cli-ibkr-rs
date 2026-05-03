# OAuth Cache Instructions

- This module owns Live Session Token cache storage and locking only.
- Redis is allowed only for centralized LST caching.
- Redis errors must fail the command; do not silently switch cache modes.
- Cache keys must not include raw consumer keys, access tokens, realms, or tokens.
