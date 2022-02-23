# A readline replacement written in Rust

![GitHub](https://img.shields.io/github/license/nushell/reedline)
[![Crates.io](https://img.shields.io/crates/v/reedline)](https://crates.io/crates/reedline)
[![docs.rs](https://img.shields.io/docsrs/reedline)](https://docs.rs/reedline/)
[![CI status](https://github.com/nushell/reedline/actions/workflows/ci.yml/badge.svg)](https://github.com/nushell/reedline/actions)
[![Discord](https://img.shields.io/discord/601130461678272522.svg?logo=discord)](https://discord.gg/NtAbbGn)
[![Twitch Status](https://img.shields.io/twitch/status/jntrnr?style=social)](https://twitch.tv/jntrnr)

## Introduction

Reedline is a project to create a readline-style crate for Rust that supports many of the modern conveniences of CLIs, including syntax highlighting, completions, multiline support, Unicode support, and more.

## Examples

### Basic example

```rust,no_run
// Create a default reedline object to handle user input

use reedline::{DefaultPrompt, Reedline, Signal};

fn main() {
    let mut line_editor = Reedline::create()?;
    let prompt = DefaultPrompt::default();

    loop {
        let sig = line_editor.read_line(&prompt).unwrap();
        match sig {
            Signal::Success(buffer) => {
                println!("We processed: {}", buffer);
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break;
            }
            Signal::CtrlL => {
                line_editor.clear_screen().unwrap();
            }
        }
    }
}
```

### Integrate with custom Keybindings

```rust,no_run
// Configure reedline with custom keybindings

//Cargo.toml
//  [dependencies]
//  crossterm = "*"

use {
  crossterm::event::{KeyCode, KeyModifiers},
  reedline::{default_emacs_keybindings, EditCommand, Reedline},
};

let mut keybindings = default_emacs_keybindings();
keybindings.add_binding(
  KeyModifiers::ALT,
  KeyCode::Char('m'),
  vec![EditCommand::BackspaceWord],
);

let mut line_editor = Reedline::create()?.with_keybindings(keybindings);
```

## Integrate with custom History

```rust,no_run
// Create a reedline object with history support, including history size limits

use reedline::{FileBackedHistory, Reedline};

let history = Box::new(
  FileBackedHistory::with_file(5, "history.txt".into())
    .expect("Error configuring history with file"),
);
let mut line_editor = Reedline::create()?
  .with_history(history)
  .expect("Error configuring reedline with history");
```

### Integrate with custom Highlighter

```rust,no_run
// Create a reedline object with highlighter support

use reedline::{ExampleHighlighter, Reedline};

let commands = vec![
  "test".into(),
  "hello world".into(),
  "hello world reedline".into(),
  "this is the reedline crate".into(),
];
let mut line_editor =
Reedline::create()?.with_highlighter(Box::new(ExampleHighlighter::new(commands)));
```

### Integrate with custom tab completion

```rust,no_run
// Create a reedline object with tab completions support

use reedline::{DefaultCompleter, Reedline, CompletionMenu};

let commands = vec![
  "test".into(),
  "hello world".into(),
  "hello world reedline".into(),
  "this is the reedline crate".into(),
];
let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));
// Use the interactive menu to select options from the completer
let completion_menu = Box::new(CompletionMenu::default());

let mut line_editor = Reedline::create()?.with_completer(completer).with_menu(completion_menu);
```

### Integrate with custom Hinter

```rust,no_run
// Create a reedline object with in-line hint support

//Cargo.toml
//  [dependencies]
//  nu-ansi-term = "*"

use {
  nu_ansi_term::{Color, Style},
  reedline::{DefaultHinter, Reedline},
};

let mut line_editor = Reedline::create()?.with_hinter(Box::new(
  DefaultHinter::default()
  .with_style(Style::new().italic().fg(Color::LightGray)),
));
```

### Integrate with custom line completion Validator

```rust,no_run
// Create a reedline object with line completion validation support

use reedline::{DefaultValidator, Reedline};

let validator = Box::new(DefaultValidator);

let mut line_editor = Reedline::create()?.with_validator(validator);
```

### Integrate with custom Edit Mode

```rust,no_run
// Create a reedline object with custom edit mode

use reedline::{EditMode, Reedline};

let mut line_editor = Reedline::create()?.with_edit_mode(
  EditMode::ViNormal, // or EditMode::Emacs or EditMode::ViInsert
);
```

## Are we prompt yet? (Development status)

This crate is currently under active development in JT's [live-coding streams](https://www.twitch.tv/jntrnr).
If you want to see a feature, jump by the streams, file an [issue](https://github.com/nushell/reedline/issues) or contribute a [PR](https://github.com/nushell/reedline/pulls)!

- [x] Basic unicode grapheme aware cursor editing.
- [x] Configurable prompt
- [x] Basic EMACS-style editing shortcuts.
- [x] Configurable keybindings.
- [x] Basic system integration with clipboard or optional stored history file.
- [x] Content aware highlighting.
- [x] Autocompletion.
- [x] Undo support.
- [x] Multiline aware editing with line completion validation.

For a more detailed roadmap check out [TODO.txt](https://github.com/nushell/reedline/blob/main/TODO.txt).

Join the vision discussion in the [vision milestone list](https://github.com/nushell/reedline/milestone/1) by contributing suggestions or voting.

### Alternatives

For currently more mature Rust line editing check out:

- [rustyline](https://crates.io/crates/rustyline)
