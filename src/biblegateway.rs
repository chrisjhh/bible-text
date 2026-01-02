use bible_data::{BOOK_ABBREVS, parse_book_abbrev};
use reqwest;
use scraper::{ElementRef, Html, Node, Selector};
use std::error::Error;
use std::result::Result;

use crate::GetChapterText;

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

pub struct BibleGateway;
impl GetChapterText for BibleGateway {
    fn get_chapter_text(
        &self,
        book: usize,
        chapter: usize,
        version: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        // Fet the text of the http response
        let text = fetch(book, chapter, version)?;

        // Get the bits we want using scraper
        // Parse as HTML
        let doc = Html::parse_document(&text);

        // Get the div containing the text
        let selector = Selector::parse("div.passage-text").unwrap();

        let div = doc.select(&selector).next().unwrap();

        // Get the paragraphs within this text
        let selector = Selector::parse("p").unwrap();

        // Other selectors we will need within the loop
        // Parse them once beforehand for efficiency
        let chapter_span = Selector::parse("span.chapternum").unwrap();
        let verse_sup = Selector::parse("sup.versenum").unwrap();
        let smallcaps_span = Selector::parse("span.small-caps").unwrap();

        // Create a mutable string to collect the text we want
        let mut chapter_text: String = String::new();

        for paragraph in div.select(&selector) {
            // Select all the text spans
            let selector = Selector::parse("span.text").unwrap();
            for span in paragraph.select(&selector) {
                for node in span.children() {
                    match node.value() {
                        Node::Text(text) => {
                            if chapter_text.ends_with(|c: char| [',', ';', '!', '.'].contains(&c))
                                && !text.starts_with(|c: char| {
                                    c.is_whitespace() || c.is_ascii_punctuation()
                                })
                            {
                                chapter_text.push_str(" ");
                            }
                            if chapter_text.ends_with(|c: char| c.is_alphanumeric())
                                && text.starts_with(|c: char| c.is_alphanumeric())
                            {
                                chapter_text.push_str(" ");
                            }
                            chapter_text.push_str(text);
                        }
                        Node::Element(_) => {
                            if let Some(element) = ElementRef::wrap(node) {
                                if chapter_span.matches(&element) {
                                    chapter_text.push_str("1 ");
                                } else if smallcaps_span.matches(&element) {
                                    chapter_text.push_str(
                                        &element.text().next().unwrap_or("").to_uppercase(),
                                    );
                                } else if verse_sup.matches(&element) {
                                    while chapter_text.ends_with(|c: char| c.is_whitespace()) {
                                        chapter_text.pop();
                                    }
                                    if chapter_text == "1" {
                                        chapter_text.pop();
                                    }
                                    if !chapter_text.is_empty() {
                                        chapter_text.push_str("\n");
                                    }
                                    chapter_text.push_str(
                                        &element
                                            .text()
                                            .next()
                                            .unwrap_or("")
                                            .replace(|c: char| c.is_whitespace(), " "),
                                    );
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(Some(chapter_text))
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
        for version in vec!["NIV", "ESV", "KJV", "NASB", "NKJV", "NLT", "HCSB"] {
            let text = bg.get_chapter_text(1, 1, version).unwrap().unwrap();
            let lines = text.lines().collect::<Vec<&str>>();
            insta::assert_yaml_snapshot!(format!("test_get_ge_1-{}", version), lines);
        }
    }

    #[test]
    fn test_get_hag_2() {
        let bg = BibleGateway;
        let book = parse_book_abbrev("Hag").unwrap();
        let text = bg.get_chapter_text(book + 1, 2, "NIV").unwrap().unwrap();
        let lines = text.lines().collect::<Vec<&str>>();
        insta::assert_yaml_snapshot!(lines);
    }

    #[test]
    fn test_get_jude() {
        let bg = BibleGateway;
        let book = parse_book_abbrev("Jude").unwrap();
        let text = bg.get_chapter_text(book + 1, 1, "KJV").unwrap().unwrap();
        let lines = text.lines().collect::<Vec<&str>>();
        insta::assert_yaml_snapshot!(lines);
    }
}
