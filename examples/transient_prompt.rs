// Create a reedline object with a transient prompt.
// cargo run --example transient_prompt
//
// Prompts for previous lines will be replaced with a shorter prompt

use nu_ansi_term::{Color, Style};
#[cfg(any(feature = "sqlite", feature = "sqlite-dynlib"))]
use reedline::SqliteBackedHistory;
use reedline::{
    ColumnarMenu, DefaultCompleter, DefaultHinter, Prompt, PromptEditMode, PromptHistorySearch,
    PromptHistorySearchStatus, Reedline, ReedlineMenu, Signal, ExampleHighlighter, default_emacs_keybindings, Keybindings, KeyModifiers, KeyCode, ReedlineEvent, Emacs, Validator, ValidationResult,
};
use std::{borrow::Cow, cell::Cell, io};

// For custom prompt, implement the Prompt trait
//
// This example replaces the prompt for old lines with "!" as an
// example of a transient prompt.
#[derive(Clone)]
pub struct TransientPrompt {
    /// Whether to show the transient prompt indicator instead of the normal one
    show_transient: Cell<bool>,
}
pub static DEFAULT_MULTILINE_INDICATOR: &str = "::: ";
pub static NORMAL_PROMPT: &str = "(transient_prompt example)";
pub static TRANSIENT_PROMPT: &str = "!";
impl Prompt for TransientPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        {
            if self.show_transient.get() {
                Cow::Owned(String::new())
            } else {
                Cow::Borrowed(NORMAL_PROMPT)
            }
        }
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Owned(String::new())
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        if self.show_transient.get() {
            Cow::Borrowed(TRANSIENT_PROMPT)
        } else {
            Cow::Owned(">".to_string())
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(DEFAULT_MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    fn repaint_on_enter(&self) -> bool {
        // This method is called whenever the user hits enter to go to the next
        // line, so we want it to repaint and display the transient prompt
        self.show_transient.set(true);
        true
    }
}

// To test multiline input. Only goes to the next line if the line ends with a ?
struct CustomValidator;

impl Validator for CustomValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.ends_with("?") {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

// This is copied from the completions example
fn add_menu_keybindings(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
}

fn main() -> io::Result<()> {
    println!("Transient prompt demo:\nAbort with Ctrl-C or Ctrl-D");
    let commands = vec![
        "test".into(),
        "hello world".into(),
        "hello world reedline".into(),
        "this is the reedline crate".into(),
    ];
    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));
    // Use the interactive menu to select options from the completer
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

    let mut keybindings = default_emacs_keybindings();
    add_menu_keybindings(&mut keybindings);

    let edit_mode = Box::new(Emacs::new(keybindings));

    let mut line_editor = Reedline::create()
        .with_hinter(Box::new(
            DefaultHinter::default().with_style(Style::new().fg(Color::LightGray)),
        ))
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode)
        .with_highlighter(Box::new(ExampleHighlighter::new(commands)))
        .with_validator(Box::new(CustomValidator {}));
    #[cfg(any(feature = "sqlite", feature = "sqlite-dynlib"))]
    {
        line_editor = line_editor.with_history(Box::new(SqliteBackedHistory::in_memory().unwrap()));
    }

    let prompt = TransientPrompt {
        show_transient: Cell::new(false),
    };

    loop {
        // We're on a new line, so make sure we're showing the normal prompt
        prompt.show_transient.set(false);
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(buffer) => {
                println!("We processed: {buffer}");
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break Ok(());
            }
        }
    }
}
