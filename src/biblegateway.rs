use super::has_attr;
use bible_data::BOOK_ABBREVS;
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest;
use std::error::Error;
use std::result::Result;

#[cfg(test)]
use super::{read_cachefile, write_cachefile};

fn fetch_internal(book: usize, chapter: usize, version: &str) -> Result<String, Box<dyn Error>> {
    if book == 0 || book > 66 {
        return Err(format!("Invalid book number. 1 <= n <= 66: {}", book).into());
    }
    if chapter == 0 || chapter > 150 {
        return Err(format!("Invalid chapter number. 1 <= n <= 150: {}", chapter).into());
    }
    let book = BOOK_ABBREVS[book - 1];
    Ok(reqwest::blocking::get(format!(
        "https://www.biblegateway.com/passage/?search={}%20{}&version={}",
        book, chapter, version
    ))?
    .text()?)
}

#[cfg(not(test))]
fn fetch(book: usize, chapter: usize, version: &str) -> Result<String, Box<dyn Error>> {
    fetch_internal(book, chapter, version)
}

#[cfg(test)]
fn fetch(book: usize, chapter: usize, version: &str) -> Result<String, Box<dyn Error>> {
    let cachefile = format!(
        "tests/cache/biblegateway/{}-{}-{}.response",
        version,
        BOOK_ABBREVS[book - 1],
        chapter
    );
    if let Ok(text) = read_cachefile(&cachefile) {
        return Ok(text);
    }
    let text = fetch_internal(book, chapter, version)?;
    write_cachefile(&cachefile, &text)?;
    Ok(text)
}

struct BibleGateway;
impl BibleGateway {
    pub fn get_chapter_text(
        &self,
        book: usize,
        chapter: usize,
        version: &str,
    ) -> Result<String, Box<dyn Error>> {
        let text = fetch(book, chapter, version)?;
        // Read block <dic class="passage-text">
        // Split on <sup class="versenum">
        let mut reader = Reader::from_str(&text);
        reader.config_mut().trim_text(true);

        let mut text: String = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"div" if has_attr(&e, b"class", "passage-text") => {
                        let mut div_depth = 1;
                        let mut span_depth = 0;
                        let mut collect_text = false;
                        let mut in_paragraph = false;
                        loop {
                            match reader.read_event() {
                                Ok(Event::Start(e)) => match e.name().as_ref() {
                                    b"span" if has_attr(&e, b"class", "text") => {
                                        collect_text = true;
                                        span_depth = 1;
                                    }
                                    b"span" if has_attr(&e, b"class", "chapternum") => {
                                        text.push_str("1 ");
                                        reader.read_to_end(e.name()).unwrap();
                                    }
                                    b"span" if collect_text => {
                                        span_depth += 1;
                                    }
                                    b"p" => in_paragraph = true,
                                    b"sup" if has_attr(&e, b"class", "versenum") => {
                                        //println!("{:?}", e);
                                        if let Ok(Event::Text(e)) = reader.read_event() {
                                            text.push_str("\n");
                                            text.push_str(
                                                &e.decode()
                                                    .unwrap()
                                                    .replace(|c: char| c.is_whitespace(), " "),
                                            );
                                        }
                                    }
                                    b"sup" => {
                                        reader.read_to_end(e.name()).unwrap();
                                    }
                                    b"div" => {
                                        //println!("{:?}", e);
                                        div_depth += 1
                                    }
                                    _ => (),
                                },
                                Ok(Event::End(e)) => match e.name().as_ref() {
                                    b"div" => {
                                        //println!("{:?}", e);
                                        div_depth -= 1;
                                        if div_depth == 0 {
                                            break;
                                        }
                                    }
                                    b"span" if collect_text => {
                                        span_depth -= 1;
                                        if span_depth == 0 {
                                            collect_text = false;
                                        }
                                    }
                                    b"p" => in_paragraph = false,
                                    _ => (),
                                },
                                Ok(Event::Text(e)) if collect_text && in_paragraph => {
                                    let verse_text = e.decode().unwrap();
                                    if !verse_text.is_empty() {
                                        // println!("Verse Text: {}", verse_text);
                                        if !text.ends_with(|c: char| c.is_whitespace())
                                            && verse_text.starts_with(|c: char| c.is_alphanumeric())
                                        {
                                            //println!("[{}] [{}]", text, verse_text);
                                            text.push_str(" ");
                                        }
                                        text.push_str(&verse_text);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                //Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_fetch() {
        let text = fetch(1, 1, "NIV");
        assert!(text.is_ok());
        assert!(Path::new("tests/cache/biblegateway/NIV-Ge-1.response").exists())
    }

    #[test]
    fn test_get_ge_1() {
        let bg = BibleGateway;
        let text = bg.get_chapter_text(1, 1, "NIV").unwrap();
        let lines = text.lines().collect::<Vec<&str>>();
        insta::assert_yaml_snapshot!(lines);
    }
}
