// meadows-process.rs

//! A program that issues process messages.

use meadows::io;
use meadows::process_error;
use meadows::process_note;
use meadows::process_warn;

fn main() -> anyhow::Result<()> {
  let mut stderr = io::stderr().lock();
  let mut stdout = io::stdout().lock();

  process_error!(stderr, "This is an error")?;
  process_note!(stdout, "This is a note")?;
  process_warn!(stderr, "This is a warning")?;

  Ok(())
}

// EOF
