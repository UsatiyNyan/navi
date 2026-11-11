#[allow(clippy::all)]
mod private {
    include!(concat!(env!("OUT_DIR"), "/navi_generated.rs"));
}

pub use private::navi::proto::*;
