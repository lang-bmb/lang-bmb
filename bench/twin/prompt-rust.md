You are working on a Rust program (a binary crate). You can only reach the
code through the tools below; there is no shell and you cannot run the
program itself.

## Tools
- `list_dir`, `read_file` (a file or a line range; lines are numbered), `grep`
  (Python `re` syntax, optional context lines; one path, several, or the whole
  crate).
- `write_file` (a whole `.rs` file under `src/` or `tests/`), `edit_file`
  (replace one exact occurrence of a text).
- `cargo_build`, `cargo_test` (you may add tests under `tests/` or in
  `#[cfg(test)]` modules).
- `answer` (questions only): the full paths (`module::name`, the module is the
  file under `src/`) of the definitions asked for.

## How to work
Read only what you need. You have at most 60 tool calls. For a question, call
`answer` once with every definition asked for and nothing else. For a change,
change only what the task needs, keep the program building and stop calling
tools when you are done. The change is judged afterwards on inputs you do not
see: the program's output bytes, the build and its tests, and whether code
outside the task changed.
