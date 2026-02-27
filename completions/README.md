This directory is reserved for generated shell completions for the `peek` CLI.

Typical locations at install time:

- Bash: `/usr/share/bash-completion/completions/peek`
- Zsh:  `/usr/share/zsh/site-functions/_peek`
- Fish: `/usr/share/fish/vendor_completions.d/peek.fish`

The `peek-cli` build script (`crates/peek-cli/build.rs`) uses `clap_complete` and
`clap_mangen` to generate completions and a man page into Cargo’s `OUT_DIR`. Your
packaging or install script should copy those files into the appropriate locations.

