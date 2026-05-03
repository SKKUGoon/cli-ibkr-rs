# OAuth Signing Instructions

- This module owns OAuth parameter encoding, nonce/timestamp generation, headers, and signatures.
- Signing code must be deterministic except for nonce and timestamp generation.
- Keep cryptographic transformations explicit and concise.
- Do not log secrets, signatures, decrypted prepends, or live session tokens.
