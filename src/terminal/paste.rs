use std::io;

use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;

pub fn enable() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnableBracketedPaste)?;
    Ok(())
}

pub fn disable() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, DisableBracketedPaste)?;
    Ok(())
}
