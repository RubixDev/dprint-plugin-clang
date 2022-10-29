# dprint-plugin-clang

Use [`clang-format`](https://clang.llvm.org/docs/ClangFormat.html) inside your
[dprint](https://dprint.dev/) config.

## Install

Add the plugin to your config file by running
`dprint config add RubixDev/clang`.

Don't forget to add the supported file extensions to your `includes` pattern.

## Configuration

This plugin uses the global dprint config keys as specified below. For further
customization,
[all options from clang-format](https://clang.llvm.org/docs/ClangFormatStyleOptions.html)
can be set through the `"clang"` config key, and the usual `.clang-format`
config files are also read unless you specify another base style through
`BasedOnStyle`.

| global dprint config key | used clang-format option |
| ------------------------ | ------------------------ |
| `newLineKind`            | `UseCRLF`                |
| `lineWidth`              | `ColumnLimit`            |
| `useTabs`                | `UseTab`                 |
| `indentWidth`            | `IndentWidth`            |
