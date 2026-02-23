use colored::Colorize;
use image::ImageError;
use image::error::*;
use posix_errors::*;

pub fn handle_image_errors(err: ImageError) -> i32 {
  let fancy_err_prefix = "error!".red().bold();

  match err {
    ImageError::Decoding(err) => {
      eprintln!("{fancy_err_prefix} decoding failed: {}", err.to_string());
      EIO
    }
    ImageError::Encoding(err) => {
      eprintln!("{fancy_err_prefix} encoding failed: {}", err.to_string());
      EIO
    }
    ImageError::IoError(err) => {
      eprintln!("{fancy_err_prefix} operation failed: {}", err.to_string());
      PosixError::from(err).code()
    }
    ImageError::Limits(err) => {
      eprintln!("{fancy_err_prefix} operation failed: {}", err.to_string());
      match err.kind() {
        LimitErrorKind::InsufficientMemory => ENOMEM,
        _ => EPERM,
      }
    }
    ImageError::Parameter(err) => {
      let err_msg: &str = match err.kind() {
        ParameterErrorKind::Generic(reason) => &reason.to_owned(),
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

      eprintln!("{fancy_err_prefix} input is invalid: {}", err_msg);
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

      eprintln!("{fancy_err_prefix} input is invalid: {}", err_msg);
      ENOTSUP
    }
  }
}
