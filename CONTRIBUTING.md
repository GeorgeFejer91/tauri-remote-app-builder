# Contributing

Open an issue or pull request with a concrete defect, protocol ambiguity, missing platform case, measured performance result, or reproducible qualification improvement.

For normative changes:

1. explain the threat or failure mode;
2. state the invariant that resolves it;
3. update the smallest relevant reference and any affected starter contract;
4. add deterministic tests;
5. run every command in the README validation block;
6. record runtime evidence separately from general claims;
7. preserve source provenance and every applicable license notice.

Do not add vendor-specific behavior to the protocol core when an adapter boundary is sufficient. Do not add a generic CLI, shell, arbitrary product-command proxy, or cryptographic construction. Authentication changes require a primary specification, maintained implementation strategy, test vectors, and an explicit migration story.
