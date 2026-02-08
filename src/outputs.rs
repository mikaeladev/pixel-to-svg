use std::{fs, io, path};

pub enum Output {
  File(fs::File),
  Stdout(io::Stdout),
}

impl io::Write for Output {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      Output::File(f) => f.write(buf),
      Output::Stdout(s) => s.write(buf),
    }
  }

  fn flush(&mut self) -> std::io::Result<()> {
    match self {
      Output::File(f) => f.flush(),
      Output::Stdout(s) => s.flush(),
    }
  }
}

impl Output {
  pub fn new(outpath: path::PathBuf) -> std::io::Result<Self> {
    if outpath.to_str() == Some("-") {
      Ok(Output::Stdout(io::stdout()))
    } else {
      Ok(Output::File(
        fs::OpenOptions::new()
          .create(true)
          .truncate(true)
          .write(true)
          .open(outpath)?,
      ))
    }
  }
}
