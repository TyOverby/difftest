use expectation_shared::{Difference, Result as EResult, ResultKind};
use std::io::{Result as IoResult, Write};

pub fn format_html<W: Write>(
    results_with_html: &[(String, Vec<EResult>, bool)],
    mut writer: W,
) -> IoResult<()> {
    write!(
        writer,
        r#"<html><head><style>{}</style></head><body>"#,
        include_str!("./style.css")
    )?;
    for (name, result, passed) in results_with_html {
        if *passed {
            continue;
        }

        write!(writer, r#"<div class="test">"#);
        write!(writer, "<h1>{}</h1>", name)?;
        write!(writer, r#"<div class="indent">"#);
        for result in result {
            match result {
                EResult {
                    kind: ResultKind::Difference(Difference { html: Some(s), .. }),
                    file_name,
                    ..
                } => {
                    write!(writer, r#"<div class="file">"#);
                    write!(writer, "<h2>{}</h2>", file_name.to_string_lossy())?;
                    write!(writer, r#"<div class="indent">"#);
                    write!(writer, "{}", s)?;
                    write!(writer, "</div>");
                    write!(writer, "</div>");
                }
                EResult {
                    kind: ResultKind::Difference(Difference { html: None, .. }),
                    file_name,
                    ..
                } => {
                    write!(writer, r#"<div class="file">"#);
                    write!(writer, "<h2>{}</h2>", file_name.to_string_lossy())?;
                    write!(writer, r#"<div class="indent">"#);
                    write!(writer, "No HTML diff for this format")?;
                    write!(writer, "</div>");
                    write!(writer, "</div>");
                }
                _ => {}
            }
        }
        write!(writer, "</div>");
        write!(writer, "</div>");
    }

    write!(writer, "</body></html>")?;
    Ok(())
}
