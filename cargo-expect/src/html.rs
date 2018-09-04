use expectation_shared::{Difference, Result as EResult, ResultKind};
use std::io::{Result as IoResult, Write};

pub fn format_html<W: Write>(
    results_with_html: &[(String, Vec<EResult>)],
    mut writer: W,
) -> IoResult<()> {
    write!(
        writer,
        r#"<html><head><style>{}</style></head><body>"#,
        include_str!("./style.css")
    )?;
    for (name, result) in results_with_html {
        write!(writer, "<h1>{}</h1>", name)?;
        for result in result {
            match result {
                EResult {
                    kind: ResultKind::Difference(Difference { html: Some(s), .. }),
                    file_name,
                    ..
                } => {
                    write!(writer, "<h2>{}</h2>", file_name.to_string_lossy())?;
                    write!(writer, "{}", s)?;
                }
                EResult {
                    kind: ResultKind::Difference(Difference { html: None, .. }),
                    ..
                } => {
                    write!(writer, "<h2>{}</h2>", name)?;
                    write!(writer, "No HTML diff for this format")?;
                }
                _ => {}
            }
        }
    }

    write!(writer, "</body></html>")?;
    Ok(())
}
