use std::process::exit;

use clap::Error as ClapError;
use image::ImageError;
use image::error::*;

pub const EPERM: i32 = 1;
pub const EIO: i32 = 5;
pub const ENOMEM: i32 = 12;
pub const EINVAL: i32 = 22;
pub const ENOTSUP: i32 = 45;

pub const ERR_PREFIX: &str = color_print::cstr!("<bold><red>error!</></>");

pub fn handle_clap_error(err: ClapError) -> ! {
  if err.use_stderr() {
    // todo: actually describe which arguments are invalid and why
    eprintln!("{ERR_PREFIX} invalid argument(s), see '--help' for usage info");
    exit(EINVAL)
  } else {
    err.exit()
  }
}

pub fn handle_image_error(err: ImageError) -> ! {
  exit(match err {
    ImageError::IoError(err) => {
      eprintln!("{ERR_PREFIX} io operation failed: {}", err.kind());
      err.raw_os_error().unwrap_or(EIO)
    }
    ImageError::Decoding(err) => {
      eprintln!("{ERR_PREFIX} decoding failed: '{err}'");
      EIO
    }
    ImageError::Encoding(err) => {
      eprintln!("{ERR_PREFIX} encoding failed: '{err}'");
      EIO
    }
    ImageError::Limits(err) => {
      eprintln!("{ERR_PREFIX} operation failed: '{err}'");
      match err.kind() {
        LimitErrorKind::InsufficientMemory => ENOMEM,
        _ => EPERM,
      }
    }
    ImageError::Parameter(err) => {
      let err_msg: &str = match err.kind() {
        ParameterErrorKind::Generic(reason) => &format!("'{reason}'"),
        ParameterErrorKind::DimensionMismatch => "dimension mismatch",
        ParameterErrorKind::NoMoreData => "malformed data",
        ParameterErrorKind::FailedAlready => "failed already",
        ParameterErrorKind::RgbCicpRequired(_) => "rgb cicp is required",
        ParameterErrorKind::CicpMismatch {
          expected: _,
          found: _,
        } => "cicp mismatch",
        _ => "unknown error",
      };

      eprintln!("{ERR_PREFIX} input is invalid: {err_msg}");
      EINVAL
    }
    ImageError::Unsupported(err) => {
      let err_msg: &str = match err.kind() {
        UnsupportedErrorKind::Color(_) => "unsupported colour type",
        UnsupportedErrorKind::ColorLayout(_) => "unsupported colour layout",
        UnsupportedErrorKind::ColorspaceCicp(_) => "unsupported colour space",
        UnsupportedErrorKind::Format(_) => "unsupported format",
        _ => "unsupported operation",
      };

      eprintln!("{ERR_PREFIX} input is invalid: {err_msg}");
      ENOTSUP
    }
  })
}
