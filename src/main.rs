use std::{
  collections::HashMap,
  error::Error,
  fs::{File, OpenOptions},
  io::{Stdout, Write, stdout},
  path::PathBuf,
};

use svg::{
  Document, Node,
  node::element::{Group, Path, Rectangle, path::Data},
};

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
  /// Path to the input file
  input: PathBuf,
  /// Path to the output file or '-' for stdout
  #[arg(short = 'O', long = "output")]
  output: Option<PathBuf>,
  /// Method used to represent pixels
  #[arg(short = 'm', long = "method", default_value = "polygons")]
  method: Method,
}

#[derive(Clone, Copy, ValueEnum)]
enum Method {
  /// Uses `<path>` elements to draw connected shapes out of line segments;
  /// results in a smaller file size and more efficient rendering
  Polygons,
  /// Uses `<rect>` elements to represent individual pixels; results in a much
  /// larger file size, but can be useful if you want to import the SVG in a
  /// pixel art editor
  Rectangles,
}

enum Output {
  File(File),
  Stdout(Stdout),
}

impl Write for Output {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
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
  fn new(path: PathBuf) -> std::io::Result<Self> {
    if path.to_str() == Some("-") {
      Ok(Output::Stdout(stdout()))
    } else {
      Ok(Output::File(
        OpenOptions::new()
          .create(true)
          .truncate(true)
          .write(true)
          .open(path)?,
      ))
    }
  }
}

fn to_hexcode(colours: &[u8]) -> String {
  let rgb = format!("#{:02x}{:02x}{:02x}", colours[0], colours[1], colours[2]);

  if colours[3] < 0xFF {
    return format!("{rgb}{:02x}", colours[3]);
  } else {
    return rgb;
  }
}

fn to_px(value: u32) -> String {
  format!("{value}px")
}

fn to_path_element(hexcode: &str, coords: &Vec<(u32, u32)>) -> Path {
  let mut data = Data::new();

  for (x, y) in coords {
    let (x, y) = (*x, *y);

    data = data
      .move_to((x, y))
      .line_to((x + 1, y))
      .line_to((x + 1, y + 1))
      .line_to((x, y + 1));
  }

  data = data.close();

  Path::new().set("fill", hexcode).set("d", data)
}

fn to_rect_element(hexcode: &str, coords: &Vec<(u32, u32)>) -> Group {
  let mut group = Group::new();

  for (x, y) in coords {
    let (x, y) = (*x, *y);

    let rect = Rectangle::new()
      .set("x", to_px(x))
      .set("y", to_px(y))
      .set("width", to_px(1))
      .set("height", to_px(1))
      .set("fill", hexcode);

    group.append(rect);
  }

  group
}

fn main() -> Result<(), Box<dyn Error>> {
  let Args {
    input,
    output,
    method,
  } = Args::parse();

  let output = output.unwrap_or_else(|| input.with_extension("svg"));

  let image_file = image::open(input)?;
  let image_data = image_file.to_rgba8();

  let pixel_data = image_data.enumerate_pixels().filter(|(_, _, c)| c[3] > 0);

  let mut pixel_map: HashMap<String, Vec<(u32, u32)>> = HashMap::new();

  for (x, y, c) in pixel_data {
    let hexcode = to_hexcode(&c.0);
    let coords = pixel_map.entry(hexcode).or_default();

    coords.push((x, y));
  }

  let mut document = Document::new()
    .set("viewBox", (0, 0, image_data.width(), image_data.height()));

  for (hexcode, coords) in pixel_map.iter() {
    match method {
      Method::Polygons => document.append(to_path_element(hexcode, coords)),
      Method::Rectangles => document.append(to_rect_element(hexcode, coords)),
    }
  }

  svg::write(Output::new(output)?, &document)?;
  Ok(())
}
