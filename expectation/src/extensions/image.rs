use super::super::provider::{Provider, WriteRequester};
use super::super::*;
use expectation_shared::filesystem::ReadSeek;

use std::io::{BufReader, Result as IoResult};
use std::path::Path;

use image::*;

pub trait ImageDiffExtension {
    fn png_writer<N>(&self, filename: N) -> Writer
    where
        N: AsRef<Path>;

    fn rgb_image<N>(&self, filename: N, image: RgbImage) -> IoResult<()>
    where
        N: AsRef<Path>,
    {
        let mut w = self.png_writer(filename);
        let dyn_image = DynamicImage::ImageRgb8(image);
        dyn_image.write_to(&mut w, ImageOutputFormat::PNG).unwrap();
        Ok(())
    }

    fn rgba_image<N>(&self, filename: N, image: RgbaImage) -> IoResult<()>
    where
        N: AsRef<Path>,
    {
        let mut w = self.png_writer(filename);
        let dyn_image = DynamicImage::ImageRgba8(image);
        dyn_image.write_to(&mut w, ImageOutputFormat::PNG).unwrap();
        Ok(())
    }
}

impl ImageDiffExtension for Provider {
    fn png_writer<S>(&self, filename: S) -> Writer
    where
        S: AsRef<Path>,
    {
        self.custom_test(
            filename,
            |a, b| image_eq(a, b),
            |a, b, c, d| image_diff(a, b, c, d),
        )
    }
}

fn image_eq<R1: ReadSeek, R2: ReadSeek>(r1: R1, r2: R2) -> IoResult<bool> {
    let mut r1 = BufReader::new(r1);
    let mut r2 = BufReader::new(r2);

    let i1 = load(&mut r1, ImageFormat::PNG).unwrap();
    let i2 = load(&mut r2, ImageFormat::PNG).unwrap();

    match (i1, i2) {
        (DynamicImage::ImageRgb8(i1), DynamicImage::ImageRgb8(i2)) => {
            if i1.width() != i2.width() || i1.height() != i2.height() {
                return Ok(false);
            }
            for x in 0..i1.width() {
                for y in 0..i1.height() {
                    if i1.get_pixel(x, y) != i2.get_pixel(x, y) {
                        return Ok(false);
                    }
                }
            }
        }
        (DynamicImage::ImageRgba8(i1), DynamicImage::ImageRgba8(i2)) => {
            if i1.width() != i2.width() || i1.height() != i2.height() {
                return Ok(false);
            }
            for x in 0..i1.width() {
                for y in 0..i1.height() {
                    if i1.get_pixel(x, y) != i2.get_pixel(x, y) {
                        return Ok(false);
                    }
                }
            }
        }
        (_, _) => return Ok(false),
    }

    Ok(true)
}

fn _add_extension(p: &Path, new_ext: &str) -> PathBuf {
    let old_ext = match p.extension() {
        Some(e) => e.to_string_lossy().into_owned(),
        None => "".to_owned(),
    };
    p.with_extension(format!("{}{}", old_ext, new_ext))
}

fn image_diff<R1: ReadSeek, R2: ReadSeek>(
    r1: R1,
    r2: R2,
    path: &Path,
    write_requester: &mut WriteRequester,
) -> IoResult<()> {
    use image::{ImageBuffer, Rgb};
    let mut r1 = BufReader::new(r1);
    let mut r2 = BufReader::new(r2);

    let i1 = load(&mut r1, ImageFormat::PNG).unwrap();
    let i2 = load(&mut r2, ImageFormat::PNG).unwrap();

    write_requester.set_html_renderer(|_, _, diffs| {
        let mut image_diff = diffs
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(".png"));
        let first = image_diff.next().unwrap();
        return format!(
            r#"
        <h3> Actual / Expected / Diff </h3>
        <img src="{}"/>
        "#,
            first.to_string_lossy()
        );
    });

    match (i1, i2) {
        (DynamicImage::ImageRgb8(i1), DynamicImage::ImageRgb8(i2)) => {
            let (w, h) = (i1.width().min(i2.width()), i1.height().min(i2.height()));
            if i1.width() != i2.width() || i1.height() != i2.height() {
                write_requester.request(path.join("img-size.txt"), |w| {
                    writeln!(w, "image dimensions are different")?;
                    writeln!(w, "actual:   width: {} height: {}", i1.width(), i1.height())?;
                    writeln!(w, "expected: width: {} height: {}", i2.width(), i2.height())?;
                    Ok(())
                })?;
            }
            let mut color_buffer: ImageBuffer<Rgb<u8>, _> =
                ImageBuffer::new(i1.width() + i2.width() + w, i1.height().max(i2.height()));

            assert!(color_buffer.copy_from(&i1, 0, 0));
            assert!(color_buffer.copy_from(&i2, i1.width(), 0));

            for x in 0..w {
                for y in 0..h {
                    let p1 = i1.get_pixel(x, y);
                    let p2 = i2.get_pixel(x, y);
                    // TODO: is this the right direction?
                    let pd = Rgb {
                        data: [
                            u8::wrapping_sub(p1[0], p2[0]),
                            u8::wrapping_sub(p1[1], p2[1]),
                            u8::wrapping_sub(p1[2], p2[2]),
                        ],
                    };
                    color_buffer.put_pixel(i1.width() + i2.width() + x, y, pd);
                }
            }
            let image = DynamicImage::ImageRgb8(color_buffer);
            return write_requester.request(path.join("color-diff.png"), |mut w| {
                image.write_to(&mut w, ImageOutputFormat::PNG).unwrap();
                Ok(())
            });
        }
        (DynamicImage::ImageRgba8(i1), DynamicImage::ImageRgba8(i2)) => {
            let (w, h) = (i1.width().min(i2.width()), i1.height().min(i2.height()));
            if i1.width() != i2.width() || i1.height() != i2.height() {
                write_requester.request(path.join("img-size.txt"), |w| {
                    writeln!(w, "image dimensions are different")?;
                    writeln!(w, "actual:   width: {} height: {}", i1.width(), i1.height())?;
                    writeln!(w, "expected: width: {} height: {}", i2.width(), i2.height())?;
                    Ok(())
                })?;
            }
            let mut color_buffer: ImageBuffer<Rgb<u8>, _> =
                ImageBuffer::new(w * 3, i1.height().max(i2.height()));
            let mut transparency_buffer: ImageBuffer<Rgb<u8>, _> =
                ImageBuffer::new(w * 3, i1.height().max(i2.height()));
            for x in 0..w {
                for y in 0..h {
                    let p1 = i1.get_pixel(x, y);
                    let p2 = i2.get_pixel(x, y);
                    // TODO: is this the right direction?
                    let pd = Rgb {
                        data: [
                            u8::wrapping_sub(p1[0], p2[0]),
                            u8::wrapping_sub(p1[1], p2[1]),
                            u8::wrapping_sub(p1[2], p2[2]),
                        ],
                    };
                    color_buffer.put_pixel(x + w + w, y, pd);
                    let pd = u8::wrapping_sub(p1[3], p2[3]);
                    transparency_buffer.put_pixel(x, y, Rgb { data: [pd, pd, pd] });
                }
            }
            let image = DynamicImage::ImageRgb8(color_buffer);
            write_requester.request(path.join("color-diff.png"), |mut w| {
                image.write_to(&mut w, ImageOutputFormat::PNG).unwrap();
                Ok(())
            })?;
            let image = DynamicImage::ImageRgb8(transparency_buffer);
            return write_requester.request(path.join("transparency-diff.png"), |mut w| {
                image.write_to(&mut w, ImageOutputFormat::PNG).unwrap();
                Ok(())
            });
        }
        (DynamicImage::ImageRgb8(_), DynamicImage::ImageRgba8(_)) => {
            return write_requester.request(path.join("img-format.txt"), |w| {
                writeln!(w, "image formats are different");
                writeln!(w, "actual:   RGB8");
                writeln!(w, "expected: RGBA8 (Alpha)");
                Ok(())
            });
        }
        (DynamicImage::ImageRgba8(_), DynamicImage::ImageRgb8(_)) => {
            return write_requester.request(path.join("img-format.txt"), |w| {
                writeln!(w, "image formats are different");
                writeln!(w, "actual:   RGBA8 (Alpha)");
                writeln!(w, "expected: RGB8");
                Ok(())
            });
        }
        (_, _) => panic!(),
    }
}
