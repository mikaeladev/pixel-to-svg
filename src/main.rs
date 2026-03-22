mod errors;

use std::{fmt, io::Write};

use clap::{Parser, ValueEnum};
use image::{EncodableLayout, ImageError, RgbaImage};
use indexmap::{IndexMap, map::IntoIter};
use patharg::{InputArg, OutputArg};

const HELP_MESSAGE: &str = color_print::cstr!(
  "\
<s><u>usage:</></> pixel-to-svg [OPTIONS] <<INPUT>>

<s><u>arguments:</></>
  <y><<INPUT>></>
    path to the input file (or '-' for stdin)

    <s>supported image formats:</>
    <dim>*</> BMP <dim>(.bmp, .dib)</>
    <dim>*</> ICO <dim>(.ico)</>
    <dim>*</> PNG <dim>(.png)</>
    <dim>*</> PNM <dim>(.pbm, .pgm, .ppm, .pnm)</>
    <dim>*</> TGA <dim>(.tga, .icb, .vda, .vst)</>

<s><u>options:</></>
  <y>-O --output <<OUTPUT>></>
    path to the output file (or '-' for stdout)

    [default: -]

  <y>-m --method <<METHOD>></>
    method used to generate the svg

    <s>possible values</>:
    <dim>*</> polygons: uses '<<path>>' elements to draw connected shapes
    <dim>*</> pixels:   uses '<<rect>>' elements to plot individual pixels

    [default: polygons]

  <y>-h --help</>
    print this help message

  <y>-V --version</>
    print the version

<s><u>about:</></>
  pixel-to-svg is a tool for converting bitmap images to scalable vectors, with
  a focus on pixel-perfection and tiny results

  <i>made with <<3 by mikaeladev <<mikaeladev@icloud.com>></>
  <i>source code is available on github <dim>(GPL-3.0)</></>
"
);

#[derive(Clone, Copy, ValueEnum)]
enum Method {
  Polygons,
  Pixels,
}

#[derive(Parser)]
#[command(about, version, override_help = HELP_MESSAGE)]
struct Args {
  input: InputArg,

  #[arg(short = 'O', long = "output", default_value = "-")]
  output: OutputArg,

  #[arg(short = 'm', long = "method", default_value = "polygons")]
  method: Method,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Coords {
  pub x: u32,
  pub y: u32,
}

impl Coords {
  pub fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }
}

impl fmt::Display for Coords {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{},{}", self.x, self.y)
  }
}

#[derive(Clone, Copy)]
pub struct ShapeData {
  pub tl: Coords,
  pub tr: Coords,
  pub bl: Coords,
  pub br: Coords,
}

impl ShapeData {
  pub fn new(coords: Coords) -> Self {
    let Coords { x, y } = coords;

    ShapeData {
      tl: Coords::new(x, y),
      tr: Coords::new(x + 1, y),
      bl: Coords::new(x, y + 1),
      br: Coords::new(x + 1, y + 1),
    }
  }

  pub fn is_adjacent(&self, other: &Self) -> bool {
    self.tl.y == other.tl.y && self.tr.x == other.tl.x
  }
}

fn to_hexcode(colours: [u8; 4]) -> String {
  let rgb = format!("#{:02x}{:02x}{:02x}", colours[0], colours[1], colours[2]);

  if colours[3] < 0xFF {
    return format!("{rgb}{:02x}", colours[3]);
  } else {
    return rgb;
  }
}

fn image_to_shapes(
  image_data: RgbaImage,
  method: &Method,
) -> IntoIter<String, Vec<ShapeData>> {
  let image_pixels = image_data.enumerate_pixels().filter(|(_, _, c)| c[3] > 0);

  let mut shape_data = IndexMap::<String, Vec<ShapeData>>::new();

  for (x, y, c) in image_pixels {
    let coords = Coords::new(x, y);
    let shape = ShapeData::new(coords);
    let hexcode = to_hexcode(c.0);

    shape_data
      .entry(hexcode)
      .and_modify(|vec| {
        let new_shape = match method {
          Method::Pixels => shape,
          Method::Polygons => match vec.pop_if(|s| s.is_adjacent(&shape)) {
            None => shape,
            Some(s) => {
              let mut new_shape = shape;
              new_shape.tl.x = s.tl.x;
              new_shape.bl.x = s.bl.x;
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

fn shapes_to_children(
  shape_data: IntoIter<String, Vec<ShapeData>>,
  method: &Method,
) -> Vec<String> {
  let mut children: Vec<String> = vec![];

  match method {
    Method::Pixels => {
      for (hexcode, shapes) in shape_data {
        for shape in shapes {
          children.push(format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
            shape.tl.x,
            shape.tl.y,
            shape.tr.x - shape.tl.x,
            shape.bl.y - shape.tl.y,
            hexcode
          ));
        }
      }
    }
    Method::Polygons => {
      for (hexcode, shapes) in shape_data {
        let mut data = String::new();

        for shape in shapes {
          let ShapeData { tl, tr, br, bl } = shape;
          data += &format!("M{tl} {tr} {br} {bl} ");
        }

        data.push('z');
        children.push(format!(r#"<path d="{data}" fill="{hexcode}" />"#))
      }
    }
  };

  children
}

fn try_main(args: Args) -> Result<(), ImageError> {
  let mut output = args.output.create()?;

  let input = match args.input {
    InputArg::Path(value) => image::open(value),
    InputArg::Stdin => image::load_from_memory(args.input.read()?.as_bytes()),
  }?;

  let image_data = input.to_rgba8();
  let image_dimensions = image_data.dimensions();

  let shapes = image_to_shapes(image_data, &args.method);
  let children = shapes_to_children(shapes, &args.method);

  writeln!(
    output,
    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">{}</svg>"#,
    image_dimensions.0,
    image_dimensions.1,
    children.join("")
  )?;

  Ok(())
}

fn main() {
  let args = match Args::try_parse() {
    Ok(a) => a,
    Err(err) => errors::handle_clap_error(err),
  };

  if let Err(err) = try_main(args) {
    errors::handle_image_error(err)
  }
}
