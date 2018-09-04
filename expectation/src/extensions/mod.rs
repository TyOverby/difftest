#[cfg(feature = "text")]
mod text;
#[cfg(feature = "text")]
pub use self::text::*;

#[cfg(feature = "image")]
mod image;
#[cfg(feature = "image")]
pub use self::image::*;

pub(crate) fn escape_html(input: &str) -> String {
    use marksman_escape::Escape;
    String::from_utf8(Escape::new(input.bytes()).collect()).unwrap()
}
