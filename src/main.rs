mod pixels;

use clap::{Parser, ValueEnum};
use image::EncodableLayout;
use patharg::{InputArg, OutputArg};
use pixels::{PixelData, ShapeData};
use std::{collections::HashMap, error::Error};
use svg::{Document, Node, node::element};

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
  #[arg(short = 'O', long = "output")]
  output: Option<OutputArg>,

  /// Method used to generate the image
  #[arg(short = 'm', long = "method", default_value = "polygons")]
  method: Method,
}

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();

  let input = match args.input {
    InputArg::Stdin => image::load_from_memory(args.input.read()?.as_bytes()),
    InputArg::Path(value) => image::open(value),
  }?;

  let output = args.output.unwrap_or_default().create()?;

  let image_data = input.to_rgba8();
  let image_pixels = image_data.enumerate_pixels().filter(|(_, _, c)| c[3] > 0);

  let mut shape_data: HashMap<String, Vec<ShapeData>> = HashMap::new();

  for (x, y, c) in image_pixels {
    let pixel = PixelData::new(x, y, c);
    let shape = ShapeData::new(&pixel);

    shape_data
      .entry(pixel.hexcode)
      .and_modify(|vec| {
        let new_shape = match args.method {
          Method::Pixels => shape,
          Method::Polygons => {
            let last_shape = vec.pop_if(|s| s.is_adjacent(&shape));

            match last_shape {
              None => shape,
              Some(s) => {
                let mut new_shape = shape;

                new_shape.tl.0 = s.tl.0;
                new_shape.bl.0 = s.bl.0;
                new_shape
              }
            }
          }
        };

        vec.push(new_shape)
      })
      .or_insert(vec![shape]);
  }

  let mut document = Document::new()
    .set("viewBox", (0, 0, image_data.width(), image_data.height()));

  for (hexcode, shapes) in shape_data {
    match args.method {
      Method::Polygons => {
        let mut path = element::Path::new();
        let mut data = element::path::Data::new();

        for shape in shapes {
          data = shape.to_polygon(data);
        }

        data = data.close();
        path = path.set("fill", hexcode).set("d", data);

        document.append(path)
      }
      Method::Pixels => {
        for shape in shapes {
          let mut rect = element::Rectangle::new();

          rect = rect.set("fill", hexcode.to_owned());
          rect = shape.to_rectangle(rect);

          document.append(rect);
        }
      }
    };
  }

  svg::write(output, &document)?;
  Ok(())
}
