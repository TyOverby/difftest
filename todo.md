
# High Priority
- [x] `cargo expect promote` 
  - [x] Acquire full paths for all files.
- [x] Redo API surface to not be so janky
- [x] Get something like concat-idents working for the stable testing macro
  - > Not needed anymore with the newer macro
- [ ] Better support for "cargo test" failing
  - [ ] Failure happened during build (`cargo check --tests` should suffice)
  - [ ] Failure happened during test (cargo-expect should at least show that a test failed...)
- [ ] Good readme
- [ ] Better Provider API
  - [x] Clone-able
  - [x] "Sub-Directory" able
  - [ ] `Send`/`Sync` support

# Mid Priority
- [ ] Sort the files that had conflicts before printing (stabilize order)
- [x] Dont write to file if the Writer was never written to.
  - [x] Maybe this isn't such a great idea?  Switch this back
- [ ] Account for multiple tests having the same name (in different modules)
  - [ ] std::module_path
- [ ] `cargo expect clean` works
- [ ] Some form of `.gitignore` support for helping people out.
- [ ] Write some epectation tests for cargo-expect
- [ ] Add serializer support
  - [ ] Ron
  - [ ] Json
- [ ] Add binary file support
- [ ] Add image file support
- [x] Find out which order (expected, actual) vs (actual, expected) the tests should be presented in.
  - > It should be (expectd, actual)
- [ ] Emit warnings when a single file has been written to more than once in the same test.
- [x] Add color to cargo-expect output

# Low Priority
- [ ] "inline" formatting for path printing
- [ ] Support handling of more cargo test command line arguments
  - [ ] `--release`
- [ ] Web "site"
  - [ ] `cargo expect browse`
  - [ ] Show passing / failing tests
  - [ ] Show diffs in the website
  - [ ] No server necessary
  - [ ] Inline button for "rebaseline"
    - [ ] This does not rebaseline immediately, it will copy commands into
      a textbox that you can copy into the command line to actually
      perform the rebaseline
    - [ ] Options to rebaseline an individual diff or a whole group.
