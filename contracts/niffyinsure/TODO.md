# Fix #31: Pure premium math refactor

Current branch: blackboxai/fix-31-pure-premium

## Steps:
- [ ] 1. Update Cargo.toml: Add serde_json, csv to [dev-dependencies]
- [ ] 2. Create src/premium_pure.rs: Pure functions/structs
- [ ] 3. Refactor src/policy.rs: Use pure functions
- [ ] 4. Deprecate/update src/premium.rs
- [ ] 5. Create tests/premium_table_tests.rs: JSON-driven golden vectors + docs
- [ ] 6. Update tests/quote.rs: Test pure paths
- [ ] 7. cargo check
- [ ] 8. Commit changes

No tests run per instructions.
