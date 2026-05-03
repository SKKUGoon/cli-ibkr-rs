# REST Instructions

- Sign every protected IBKR request with the OAuth live session token.
- Return non-2xx IBKR responses as errors that include status and body.
- Do not add retry or fallback behavior in this layer.

