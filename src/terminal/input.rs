use crossterm::event::{self, Event, KeyEvent};

pub fn poll_event(timeout_ms: u16) -> Result<Option<Event>, std::io::Error> {
    if event::poll(std::time::Duration::from_millis(timeout_ms as u64))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn read_key() -> Result<KeyEvent, std::io::Error> {
    loop {
        let event = event::read()?;
        if let Event::Key(key) = event {
            return Ok(key);
        }
    }
}
