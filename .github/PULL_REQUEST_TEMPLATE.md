## Summary

<!-- Briefly describe what this PR does and why. -->

## Checklist

- [ ] Commits follow the [Conventional Commits](https://www.conventionalcommits.org/) spec
  - `feat:` new feature
  - `fix:` bug fix
  - `docs:` documentation only
  - `chore:` maintenance / tooling
  - `ci:` CI/CD changes
  - `refactor:` code change that neither fixes a bug nor adds a feature
  - `test:` adding or correcting tests
  - `perf:` performance improvement
- [ ] `cargo test` passes locally
- [ ] `cargo clippy --all-targets -- -W clippy::pedantic` passes with no warnings
- [ ] `cargo fmt --check` passes
