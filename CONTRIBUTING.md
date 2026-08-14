# Contributing

Thanks for looking. A few things that make a change easy to take:

1. Open an issue first for anything larger than a bug fix, so we agree on the shape.
2. Keep the change focused - one behaviour per pull request.
3. Run the checks before opening the PR:
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
- for the viewer: `npm --prefix viewer test`
4. Add a test for the behaviour you changed.
5. Match the surrounding style; the codebase is deliberately small and flat.
6. Re-run `cargo run -- adjudicate samples/cache-coherence.qf` before pushing - it exercises
   the parser, the adjudicator and the report writer in one pass.
