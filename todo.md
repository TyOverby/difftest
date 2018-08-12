
# High Priority
1. `cargo expect promote`
2. Redo API surface to not be so janky
3. Get something like concat-idents working for the stable testing macro

# Mid Priority
- `cargo expect clean` works
- Some form of `.gitignore` support for helping people out.
- Write some epectation tests for cargo-expect
- Add serializer support
  - Ron
  - Json
- Add binary file support
- Add image file support
- Find out which order (expected, actual) vs (actual, expected) the tests should be presented in.

# Low Priority
- Support handling of more cargo test command line arguments
  - `--release`
- Web "site"
  - `cargo expect browse`
  - Show passing / failing tests
  - Show diffs in the website
  - No server necessary
  - Inline button for "rebaseline"
    - This does not rebaseline immediately, it will copy commands into
      a textbox that you can copy into the command line to actually
      perform the rebaseline
    - Options to rebaseline an individual diff or a whole group.
