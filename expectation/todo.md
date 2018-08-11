- Find out which order (expected, actual) vs (actual, expected) the tests should be presented in.
- Add support for file-globbing for input
- Make strategies more general purpose
  - Maybe a more general method of parsing / diffing / equality
    that defaults to just being a Vec<u8>
  - Build Skeletor support in
  - Offer option to write and read using traits rather than Vec<u8>
- Support handling of command line arguments
- Better commandline support
  - Parse args...
    - ...in order to filter tests
    - ...in order to optionally rebaseline tests
    - ...in order to be verbose or not
  - Pretty output
    - Good defaults for test reporting
    - Quiet mode
    - Verbose mode for showing textual diffs inline
- Web "site"
  - Default way to interact with the testing tool
  - Show passing / failing tests
  - Show diffs in the website
  - No server necessary
  - Inline button for "rebaseline"
    - This does not rebaseline immediately, it will copy commands into
      a textbox that you can copy into the command line to actually
      perform the rebaseline
    - Options to rebaseline an individual diff or a whole group.
