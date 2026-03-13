use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, poll},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

const LOADING_BAR_SIZE: i32 = 30;

#[derive(PartialEq)]
pub enum State {
    Active,
    Quit,
}

pub struct LoadingBar {
    pub state: State,
    pub sout: Stdout,
    pub processed_counter: i32,
    pub files_to_process: i32,
    icp_col: u16,
    icp_row: u16,
}

impl LoadingBar {
    pub fn new(sout: Stdout, files_to_process: i32, icp_col: u16, icp_row: u16) -> Self {
        Self {
            state: State::Active,
            sout,
            processed_counter: 0,
            files_to_process,
            icp_col,
            icp_row,
        }
    }

    pub fn loading_bar(&mut self) -> io::Result<()> {
        // move to init cursor pos
        self.sout.queue(MoveTo(self.icp_col, self.icp_row))?;

        // write progress text
        let text = format!("{}/{} ", self.processed_counter, self.files_to_process);
        self.sout.write(text.as_bytes())?;

        // === increment loading bar ===
        self.move_to_loading_bar_beginning(&text, 0)?;
        self.sout.write("".as_bytes())?;

        self.move_to_loading_bar_beginning(&text, 4)?;
        // fill empty parts of loading bar
        let progress = "░".repeat(LOADING_BAR_SIZE as usize - 1);
        self.sout.write(progress.as_bytes())?;

        // calculate space to fill
        let fill_n = ((self.processed_counter as f64 / self.files_to_process as f64)
            * LOADING_BAR_SIZE as f64)
            .floor();

        self.move_to_loading_bar_beginning(&text, 3)?;
        // write progress
        let progress = "█".repeat(fill_n as usize);
        self.sout.write(progress.as_bytes())?;

        self.move_to_loading_bar_beginning(&text, LOADING_BAR_SIZE as u16 + 3)?;
        self.sout.write("".as_bytes())?;
        Ok(())
    }

    /// helper: loading_bar()
    fn move_to_loading_bar_beginning(&mut self, text: &String, offset: u16) -> io::Result<()> {
        self.sout.queue(MoveTo(
            self.icp_col + text.chars().count() as u16 + offset,
            self.icp_row,
        ))?;
        Ok(())
    }

    pub fn controls(&mut self) -> std::io::Result<()> {
        if poll(Duration::ZERO)? {
            match self.state {
                State::Active => {
                    if let Event::Key(KeyEvent {
                        code, modifiers, ..
                    }) = event::read()?
                    {
                        match (code, modifiers) {
                            // quit
                            (KeyCode::Esc, _) => {
                                self.state = State::Quit;
                            }

                            (
                                KeyCode::Char('c'),
                                KeyModifiers::CONTROL,
                            ) => {
                                self.state = State::Quit;
                            }
                            _ => {}
                        }
                    }
                }

                State::Quit => {}
            }
        }
        Ok(())
    }

    pub fn util_setup(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        self.sout.queue(cursor::SavePosition)?;
        self.sout.queue(cursor::Hide)?;
        Ok(())
    }

    pub fn util_cleanup(&mut self) -> io::Result<()> {
        self.sout.queue(MoveTo(self.icp_col, self.icp_row))?;
        execute!(self.sout, Clear(ClearType::CurrentLine))?;
        disable_raw_mode()?;
        self.sout.queue(cursor::RestorePosition)?;
        self.sout.queue(MoveTo(self.icp_col, self.icp_row))?;
        self.sout.queue(cursor::Show)?;
        Ok(())
    }
}
