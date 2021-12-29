use crate::core_editor::Editor;
use crossterm::terminal::ScrollUp;

use {
    crate::{
        prompt::{PromptEditMode, PromptHistorySearch},
        Prompt,
    },
    crossterm::{
        cursor::{self, MoveTo, MoveToColumn, RestorePosition, SavePosition},
        style::{Print, ResetColor, SetForegroundColor},
        terminal::{self, Clear, ClearType},
        QueueableCommand, Result,
    },
    std::io::{Stdout, Write},
    unicode_width::UnicodeWidthStr,
};

// const END_LINE: &str = if cfg!(windows) { "\r\n" } else { "\n" };

#[derive(Default)]
struct PromptCoordinates {
    prompt_start: (u16, u16),
    input_start: (u16, u16),
}

impl PromptCoordinates {
    fn set_prompt_start(&mut self, col: u16, row: u16) {
        self.prompt_start = (col, row);
    }

    fn set_input_start(&mut self, col: u16, row: u16) {
        self.input_start = (col, row);
    }
}

pub struct PromptLines {
    before_cursor: String,
    after_cursor: String,
    hint: String,
}

impl PromptLines {
    /// Splits the strings before and after the cursor as well as the hint
    /// This vector with the str are used to calculate how many lines are
    /// required to print after the prompt
    pub fn new(before_cursor: String, after_cursor: String, hint: String) -> Self {
        Self {
            before_cursor,
            after_cursor,
            hint,
        }
    }

    fn before_cursor_lines(&self) -> u16 {
        self.before_cursor.lines().count() as u16
    }

    fn after_cursor_lines(&self) -> u16 {
        self.after_cursor.lines().count() as u16
    }

    fn hint_lines(&self) -> u16 {
        self.hint.lines().count() as u16
    }
}

pub struct Painter {
    // Stdout
    stdout: Stdout,
    prompt_coords: PromptCoordinates,
    terminal_size: (u16, u16),
}

impl Painter {
    pub fn new(stdout: Stdout) -> Self {
        Painter {
            stdout,
            prompt_coords: PromptCoordinates::default(),
            terminal_size: (0, 0),
        }
    }

    /// Calculates the how many lines are required to print all the prompt
    /// lines including the hint
    /// It also calculates the distance from the prompt which can be used
    /// as another parameter to scroll the prompt
    pub fn required_lines(&self, lines: &PromptLines) -> Result<(u16, u16)> {
        let before_cursor_lines = lines.before_cursor_lines();
        let after_cursor_lines = lines.after_cursor_lines();
        let hint_lines = lines.hint_lines();
        let delta = after_cursor_lines.max(hint_lines);

        // The minus 1 is because the delta is sharing a line with the before
        // cursor lines
        let required_lines = (before_cursor_lines + delta - 1) as u16;
        let required_lines = required_lines;

        let (_, cursor_row) = cursor::position()?;
        let distance_from_prompt = cursor_row - self.prompt_coords.prompt_start.1;
        let required_lines = required_lines.saturating_sub(distance_from_prompt);

        Ok((required_lines, distance_from_prompt))
    }

    /// Update the terminal size information by polling the system
    pub(crate) fn init_terminal_size(&mut self) -> Result<()> {
        self.terminal_size = terminal::size()?;
        Ok(())
    }

    fn terminal_columns(&self) -> u16 {
        self.terminal_size.0
    }

    fn terminal_rows(&self) -> u16 {
        self.terminal_size.1
    }

    pub fn remaining_lines(&self) -> u16 {
        self.terminal_size.1 - self.prompt_coords.prompt_start.1
    }

    /// Move cursor, doesn't flush
    pub fn queue_move_to(&mut self, column: u16, row: u16) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(column, row))?;

        Ok(())
    }

    /// Scroll by n rows
    pub fn scroll_rows(&mut self, num_rows: u16) -> Result<()> {
        self.stdout
            .queue(crossterm::terminal::ScrollUp(num_rows))?
            .flush()?;

        Ok(())
    }

    /// Sets the prompt origin position.
    pub(crate) fn initialize_prompt_position(&mut self) -> Result<()> {
        // Cursor positions are 0 based here.
        let (column, row) = cursor::position()?;
        // Assumption: if the cursor is not on the zeroth column,
        // there is content we want to leave intact, thus advance to the next row
        let new_row = if column > 0 { row + 1 } else { row };
        // TODO: support more than just two line prompts!
        //  If we are on the last line or would move beyond the last line due to
        //  the condition above, we need to make room for the multiline prompt.
        //  Otherwise printing the prompt would scroll of the stored prompt
        //  origin, causing issues after repaints.
        let new_row = if new_row == self.terminal_rows() {
            // Would exceed the terminal height, we need space for two lines
            self.print_crlf()?;
            self.print_crlf()?;
            new_row.saturating_sub(2)
        } else if new_row == self.terminal_rows() - 1 {
            // Safe on the last line make space for the 2 line prompt
            self.print_crlf()?;
            new_row.saturating_sub(1)
        } else {
            new_row
        };
        self.prompt_coords.set_prompt_start(0, new_row);
        Ok(())
    }

    pub fn repaint_everything(
        &mut self,
        prompt: &dyn Prompt,
        prompt_mode: PromptEditMode,
        lines: PromptLines,
        use_ansi_coloring: bool,
    ) -> Result<()> {
        self.stdout.queue(cursor::Hide)?;
        let (required_lines, distance) = self.required_lines(&lines)?;
        let remaining_lines = self.remaining_lines();

        if required_lines > remaining_lines {
            // Checked sub in case there is overflow
            let sub = self
                .prompt_coords
                .prompt_start
                .1
                .checked_sub(required_lines);

            if let Some(sub) = sub {
                self.stdout.queue(ScrollUp(required_lines))?;
                self.prompt_coords.prompt_start.1 = sub;
            }
        } else if (remaining_lines - distance) == 1 {
            self.stdout.queue(ScrollUp(1))?;
            self.prompt_coords.prompt_start.1 = self.prompt_coords.prompt_start.1.saturating_sub(1);
        };

        self.queue_move_to(
            self.prompt_coords.prompt_start.0,
            self.prompt_coords.prompt_start.1,
        )?;

        let (screen_width, _) = self.terminal_size;

        // print our prompt with color
        if use_ansi_coloring {
            self.stdout
                .queue(SetForegroundColor(prompt.get_prompt_color()))?;
        }

        self.stdout
            .queue(MoveToColumn(0))?
            .queue(Clear(ClearType::FromCursorDown))?
            .queue(Print(prompt.render_prompt(screen_width as usize)))?
            .queue(Print(prompt.render_prompt_indicator(prompt_mode)))?
            .queue(Print(&lines.before_cursor))?
            .queue(SavePosition)?
            .queue(Print(&lines.hint))?
            .queue(Print(&lines.after_cursor))?
            .queue(RestorePosition)?;

        if use_ansi_coloring {
            self.stdout.queue(ResetColor)?;
        }
        self.stdout.queue(cursor::Show)?;

        self.flush()
    }

    /// Updates prompt origin and offset to handle a screen resize event
    pub(crate) fn handle_resize(&mut self, width: u16, height: u16) {
        let prev_terminal_size = self.terminal_size;

        self.terminal_size = (width, height);
        // TODO properly adjusting prompt_origin on resizing while lines > 1

        let current_origin = self.prompt_coords.prompt_start;

        if current_origin.1 >= (height - 1) {
            // Terminal is shrinking up
            // FIXME: use actual prompt size at some point
            // Note: you can't just subtract the offset from the origin,
            // as we could be shrinking so fast that the offset we read back from
            // crossterm is past where it would have been.
            self.prompt_coords
                .set_prompt_start(current_origin.0, height - 2);
        } else if prev_terminal_size.1 < height {
            // Terminal is growing down, so move the prompt down the same amount to make space
            // for history that's on the screen
            // Note: if the terminal doesn't have sufficient history, this will leave a trail
            // of previous prompts currently.
            self.prompt_coords.set_prompt_start(
                current_origin.0,
                current_origin.1 + (height - prev_terminal_size.1),
            );
        }
    }

    /// TODO! FIX the naming and provide an accurate doccomment
    /// This function repaints and updates offsets but does not purely concern it self with wrapping
    // pub(crate) fn wrap(&mut self, lines: PromptLines) -> Result<()> {
    //     let (original_column, original_row) = cursor::position()?;

    //     self.queue_buffer(lines)?;

    //     let (new_column, _new_row) = cursor::position()?;

    //     if new_column < original_column && original_row + 1 == self.terminal_rows() {
    //         // We have wrapped off bottom of screen, and prompt is on new row
    //         // We need to update the prompt location in this case
    //         let (input_start_col, input_start_row) = self.prompt_coords.input_start;
    //         let (prompt_start_col, prompt_start_row) = self.prompt_coords.prompt_start;

    //         if input_start_row >= 1 {
    //             self.prompt_coords
    //                 .set_input_start(input_start_col, input_start_row - 1);
    //         } else {
    //             self.prompt_coords.set_input_start(0, 0);
    //         }

    //         if prompt_start_row >= 1 {
    //             self.prompt_coords
    //                 .set_prompt_start(prompt_start_col, prompt_start_row - 1);
    //         } else {
    //             self.prompt_coords.set_prompt_start(0, 0);
    //         }
    //     }

    //     Ok(())
    // }

    /// Heuristic to determine if we need to wrap text around.
    // pub(crate) fn require_wrapping(&self, editor: &Editor) -> bool {
    //     let line_start = if editor.line() == 0 {
    //         self.prompt_coords.input_start_col()
    //     } else {
    //         0
    //     };

    //     let terminal_width = self.terminal_columns();

    //     let display_width = UnicodeWidthStr::width(editor.get_buffer()) + line_start as usize;

    //     display_width >= terminal_width as usize
    // }

    /// Repositions the prompt offset position, if the buffer content would overflow the bottom of the screen.
    /// Checks for content that might overflow in the core buffer.
    /// Performs scrolling and updates prompt and input position.
    /// Does not trigger a full repaint!
    pub(crate) fn adjust_prompt_position(&mut self, editor: &Editor) -> Result<()> {
        let (prompt_start_col, prompt_start_row) = self.prompt_coords.prompt_start;
        let (input_start_col, input_start_row) = self.prompt_coords.input_start;

        let mut buffer_line_count = editor.num_lines() as u16;

        let terminal_columns = self.terminal_columns();

        // Estimate where we're going to wrap around the edge of the terminal
        for line in editor.get_buffer().lines() {
            let estimated_width = UnicodeWidthStr::width(line);

            let estimated_line_count = estimated_width as f64 / terminal_columns as f64;
            let estimated_line_count = estimated_line_count.ceil() as u64;

            // Any wrapping we estimate we might have, go ahead and add it to our line count
            if estimated_line_count >= 1 {
                buffer_line_count += (estimated_line_count - 1) as u16;
            }
        }

        let ends_in_newline = editor.ends_with('\n');

        let terminal_rows = self.terminal_rows();

        if input_start_row + buffer_line_count > terminal_rows {
            let spill = input_start_row + buffer_line_count - terminal_rows;

            // FIXME: see if we want this as the permanent home
            if ends_in_newline {
                self.scroll_rows(spill - 1)?;
            } else {
                self.scroll_rows(spill)?;
            }

            // We have wrapped off bottom of screen, and prompt is on new row
            // We need to update the prompt location in this case

            if spill <= input_start_row {
                self.prompt_coords
                    .set_input_start(input_start_col, input_start_row - spill);
            } else {
                self.prompt_coords.set_input_start(0, 0);
            }

            if spill <= prompt_start_row {
                self.prompt_coords
                    .set_prompt_start(prompt_start_col, prompt_start_row - spill);
            } else {
                self.prompt_coords.set_prompt_start(0, 0);
            }
        }

        Ok(())
    }

    pub fn queue_history_search_indicator(
        &mut self,
        prompt: &dyn Prompt,
        prompt_search: PromptHistorySearch,
        use_ansi_coloring: bool,
    ) -> Result<()> {
        // print search prompt
        self.stdout.queue(MoveToColumn(0))?;
        if use_ansi_coloring {
            self.stdout
                .queue(SetForegroundColor(prompt.get_prompt_color()))?;
        }
        self.stdout.queue(Print(
            prompt.render_prompt_history_search_indicator(prompt_search),
        ))?;
        if use_ansi_coloring {
            self.stdout.queue(ResetColor)?;
        }
        Ok(())
    }

    pub fn queue_history_search_result(
        &mut self,
        history_result: &str,
        offset: usize,
    ) -> Result<()> {
        self.stdout
            .queue(Print(&history_result[..offset]))?
            .queue(SavePosition)?
            .queue(Print(&history_result[offset..]))?
            .queue(Clear(ClearType::UntilNewLine))?
            .queue(RestorePosition)?;

        self.flush()
    }

    /// Writes `line` to the terminal with a following carriage return and newline
    pub fn paint_line(&mut self, line: &str) -> Result<()> {
        self.stdout
            .queue(Print(line))?
            .queue(Print("\n"))?
            .queue(MoveToColumn(1))?;

        self.stdout.flush()
    }

    /// Goes to the beginning of the next line
    ///
    /// Also works in raw mode
    pub(crate) fn print_crlf(&mut self) -> Result<()> {
        self.stdout.queue(Print("\r\n"))?;
        self.stdout.flush()
    }

    /// Clear the screen by printing enough whitespace to start the prompt or
    /// other output back at the first line of the terminal.
    pub fn clear_screen(&mut self) -> Result<()> {
        let (_, num_lines) = terminal::size()?;
        for _ in 0..2 * num_lines {
            self.stdout.queue(Print("\n"))?;
        }
        self.stdout.queue(MoveTo(0, 0))?;
        self.stdout.flush()
    }

    pub(crate) fn clear_until_newline(&mut self) -> Result<()> {
        self.stdout.queue(Clear(ClearType::UntilNewLine))?;
        self.stdout.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }
}
