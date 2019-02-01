# Templates Authoring

The documentation is a **Work In Progress**, feel free to ask details, missing info or to submit content (via Pull Request).

## Rules

- The minimal template is an empty dir.
- a sample template and its expected output (on empty folder) is available at [tests/test_1](tests/test_1).
- file priority (what file will be used if they have the same destination path)

  ```txt
  existing file
  file with source extension .ffizer.hbs (and no {{...}} in the source file path)
  file with identical source file name (and extension)
  file with {{...}} in the source file path
  ```
