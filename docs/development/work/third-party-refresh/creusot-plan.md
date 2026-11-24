# Creusot 0.15 migration

1. Pin the hoimin recipe archive by commit and checksum; align creusot-std and
   why3find solvers with its manifest. Invalidate the old image identity using
   the existing explicit bootstrap pin format.
2. Build the ARM64 development image and check installation/preflight. Run
   fresh generation, all three target proof inventories and sensitivity checks.
   Preserve failures as evidence and fix compatibility issues before claiming
   verification.
3. Update environment ownership documentation and refresh evidence. Run shell
   orchestration tests, Rust/Python checks and documentation gates.
4. Commit on the dedicated branch and open a PR against the checked remote main.
   The trusted-main publisher can publish the replacement attested image after
   merge; a PR build cannot claim trusted-main provenance. Adopt its observed
   digest separately, never invent or reuse a stale digest.
