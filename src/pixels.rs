use image::Rgba;
use svg::node::element;

fn to_hexcode(colours: &[u8]) -> String {
  let rgb = format!("#{:02x}{:02x}{:02x}", colours[0], colours[1], colours[2]);

  if colours[3] < 0xFF {
    return format!("{rgb}{:02x}", colours[3]);
  } else {
    return rgb;
  }
}

pub type Coords = (u32, u32);

#[derive(Clone)]
pub struct PixelData {
  pub hexcode: String,
  pub coords: Coords,
}

impl PixelData {
  pub fn new(x: u32, y: u32, c: &Rgba<u8>) -> PixelData {
    PixelData {
      hexcode: to_hexcode(&c.0),
      coords: (x, y),
    }
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
  pub fn new(pixel: &PixelData) -> ShapeData {
    let (x, y) = pixel.coords;

    ShapeData {
      tl: (x, y),
      tr: (x + 1, y),
      bl: (x, y + 1),
      br: (x + 1, y + 1),
    }
  }

  pub fn is_adjacent(&self, shape: &Self) -> bool {
    self.tl.1 == shape.tl.1 && self.tr.0 == shape.tl.0
  }

  pub fn to_polygon<T>(self, data: T) -> element::path::Data
  where
    T: Into<Option<element::path::Data>>,
  {
    data
      .into()
      .unwrap_or_default()
      .move_to(self.tl)
      .line_to(self.tr)
      .line_to(self.br)
      .line_to(self.bl)
  }

  pub fn to_rectangle<T>(self, rectangle: T) -> element::Rectangle
  where
    T: Into<Option<element::Rectangle>>,
  {
    rectangle
      .into()
      .unwrap_or_default()
      .set("x", self.tl.0)
      .set("y", self.tl.1)
      .set("width", self.tr.0 - self.tl.0)
      .set("height", self.bl.1 - self.tl.1)
  }
}
