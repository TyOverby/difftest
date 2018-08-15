
# High Priority
1. `cargo expect promote`
2. Redo API surface to not be so janky
   - Hide most of it with doc_hidden?
3. Get something like concat-idents working for the stable testing macro
4. Good readme

# Mid Priority
- Account for multiple tests having the same name (in different modules)
  - std::module_path
- `cargo expect clean` works
- Some form of `.gitignore` support for helping people out.
- Write some epectation tests for cargo-expect
- Add serializer support
  - Ron
  - Json
- Add binary file support
- Add image file support
- Find out which order (expected, actual) vs (actual, expected) the tests should be presented in.
- Emit warnings when a single file has been written to more than once in the same test.

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
