use {
    crossterm::{
        event::{poll, Event, KeyCode, KeyModifiers},
        terminal, Result,
    },
    nu_ansi_term::{Color, Style},
    reedline::{
        default_emacs_keybindings, DefaultCompleter, DefaultCompletionActionHandler,
        DefaultHighlighter, DefaultHinter, DefaultPrompt, EditCommand, FileBackedHistory, Reedline,
        Signal,
    },
    std::{
        io::{stdout, Write},
        time::Duration,
    },
};

fn main() -> Result<()> {
    let vi_mode = matches!(std::env::args().nth(1), Some(x) if x == "--vi");
    // quick command like parameter handling
    let args: Vec<String> = std::env::args().collect();
    // if -k is passed, show the events
    if args.len() > 1 && args[1] == "-k" {
        println!("Ready to print events:");
        print_events()?;
        println!();
        return Ok(());
    };

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('m'),
        vec![EditCommand::BackspaceWord],
    );

    let history = Box::new(FileBackedHistory::with_file(50, "history.txt".into())?);
    let commands = vec![
        "test".into(),
        "clear".into(),
        "exit".into(),
        "history".into(),
        "logout".into(),
        "hello world".into(),
        "hello world reedline".into(),
        "this is the reedline crate".into(),
    ];

    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 2));

    let mut line_editor = Reedline::new()?
        .with_history(history)?
        .with_edit_mode(if vi_mode {
            reedline::EditMode::ViNormal
        } else {
            reedline::EditMode::Emacs
        })
        .with_keybindings(keybindings)
        .with_highlighter(Box::new(DefaultHighlighter::new(commands)))
        .with_completion_action_handler(Box::new(
            DefaultCompletionActionHandler::default().with_completer(completer.clone()),
        ))
        .with_hinter(Box::new(
            DefaultHinter::default()
                .with_completer(completer) // or .with_history()
                // .with_inside_line()
                .with_style(Style::new().italic().fg(Color::LightGray)),
        ));

    let prompt = DefaultPrompt::new(1);

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::CtrlD) => {
                break;
            }
            Ok(Signal::Success(buffer)) => {
                if (buffer.trim() == "exit") || (buffer.trim() == "logout") {
                    break;
                }
                if buffer.trim() == "clear" {
                    line_editor.clear_screen()?;
                    continue;
                }
                if buffer.trim() == "history" {
                    line_editor.print_history()?;
                    continue;
                }
                line_editor.print_line(&format!("Our buffer: {}", buffer))?;
            }
            Ok(Signal::CtrlC) => {
                // We need to move one line down to start with the prompt on a new line
                line_editor.print_crlf()?;
            }
            Ok(Signal::CtrlL) => {
                line_editor.clear_screen()?;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    println!();
    Ok(())
}

/// **For debugging purposes only:** Track the terminal events observed by [`Reedline`] and print them.
pub fn print_events() -> Result<()> {
    stdout().flush()?;
    terminal::enable_raw_mode()?;
    let result = print_events_helper();
    terminal::disable_raw_mode()?;

    result
}

// this fn is totally ripped off from crossterm's examples
// it's really a diagnostic routine to see if crossterm is
// even seeing the events. if you press a key and no events
// are printed, it's a good chance your terminal is eating
// those events.
fn print_events_helper() -> Result<()> {
    loop {
        // Wait up to 5s for another event
        if poll(Duration::from_millis(5_000))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = crossterm::event::read()?;

            // just reuse the print_message fn to show events
            println!("Event::{:?}\r", event);

            // hit the esc key to git out
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 5s
            println!("Waiting for you to type...\r");
        }
    }

    Ok(())
}
