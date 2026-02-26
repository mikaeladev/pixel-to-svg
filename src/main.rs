mod errors;
mod pixels;

use std::{
  collections::{HashMap, hash_map::IntoIter},
  process::exit,
};

use clap::{Parser, ValueEnum};
use image::{EncodableLayout, ImageError, RgbaImage};
use patharg::{InputArg, OutputArg};
use pixels::{PixelData, ShapeData};
use svg::{Document, Node};

#[derive(Clone, Copy, ValueEnum)]
enum Method {
  /// Uses `<path>` elements to draw connected shapes out of line segments;
  /// results in a small file size and efficient rendering (recommended)
  Polygons,
  /// Uses `<rect>` elements to plot individual pixels one by one; results in a
  /// much larger file size, but an easily editable result
  Pixels,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
  /// Path to the input file or '-' for stdin
  input: InputArg,

  /// Path to the output file or '-' for stdout
  #[arg(short = 'O', long = "output", default_value = "-")]
  output: OutputArg,

  /// Method used to generate the image
  #[arg(short = 'm', long = "method", default_value = "polygons")]
  method: Method,
}

fn image_to_shapes(
  image_data: &RgbaImage,
  method: &Method,
) -> IntoIter<String, Vec<ShapeData>> {
  let image_pixels = image_data.enumerate_pixels().filter(|(_, _, c)| c[3] > 0);

  let mut shape_data: HashMap<String, Vec<ShapeData>> = HashMap::new();

  for (x, y, c) in image_pixels {
    let pixel = PixelData::new(x, y, c);
    let shape = ShapeData::new(&pixel);

    shape_data
      .entry(pixel.hexcode)
      .and_modify(|vec| {
        let new_shape = match method {
          Method::Pixels => shape,
          Method::Polygons => match vec.pop_if(|s| s.is_adjacent(&shape)) {
            None => shape,
            Some(s) => {
              let mut new_shape = shape;
              new_shape.tl.0 = s.tl.0;
              new_shape.bl.0 = s.bl.0;
              new_shape
            }
          },
        };

        vec.push(new_shape)
      })
      .or_insert(vec![shape]);
  }

  shape_data.into_iter()
}

fn shapes_to_document(
  image_data: &RgbaImage,
  shape_data: IntoIter<String, Vec<ShapeData>>,
  method: &Method,
) -> Document {
  use svg::node::element::{Path, Rectangle, path::Data as PathData};

  let (width, height) = image_data.dimensions();

  let mut document = Document::new().set("viewBox", (0, 0, width, height));

  for (hexcode, shapes) in shape_data {
    match method {
      Method::Pixels => {
        for shape in shapes {
          let mut rect = Rectangle::new();

          rect = rect.set("fill", hexcode.to_owned());
          rect = shape.to_rectangle(rect);

          document.append(rect);
        }
      }
      Method::Polygons => {
        let mut path = Path::new();
        let mut data = PathData::new();

        for shape in shapes {
          data = shape.to_polygon(data);
        }

        data = data.close();
        path = path.set("fill", hexcode).set("d", data);

        document.append(path)
      }
    };
  }

  document
}

fn try_main() -> Result<(), ImageError> {
  let args = Args::parse();

  let output = args.output.create()?;

  let input = match args.input {
    InputArg::Path(value) => image::open(value),
    InputArg::Stdin => image::load_from_memory(args.input.read()?.as_bytes()),
  }?;

  let image_data = input.to_rgba8();

  let shapes = image_to_shapes(&image_data, &args.method);
  let document = shapes_to_document(&image_data, shapes, &args.method);

  svg::write(output, &document)?;

  Ok(())
}

fn main() {
  exit(match try_main() {
    Ok(_) => 0,
    Err(err) => errors::handle_image_errors(err),
  })
}
