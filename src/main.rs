use reqwest;
use scraper::Node;
use scraper::{ElementRef, Html, Selector};

fn main() {
    let text =
        reqwest::blocking::get("https://www.biblegateway.com/passage/?search=ge%201&version=KJV")
            .unwrap()
            .text()
            .unwrap();

    // Parse as HTML
    let doc = Html::parse_document(&text);

    // Get the div containing the text
    let selector = Selector::parse("div.passage-text").unwrap();

    let div = doc.select(&selector).next().unwrap();

    // Get the paragraphs within this text
    let selector = Selector::parse("p").unwrap();

    let mut chapter_text: String = String::new();

    for paragraph in div.select(&selector) {
        // Select all the text spans
        let selector = Selector::parse("span.text").unwrap();
        for span in paragraph.select(&selector) {
            for node in span.children() {
                let chapter_span = Selector::parse("span.chapternum").unwrap();
                let verse_sup = Selector::parse("sup.versenum").unwrap();
                match node.value() {
                    Node::Text(text) => {
                        chapter_text.push_str(text);
                    }
                    Node::Element(_) => {
                        if let Some(element) = ElementRef::wrap(node) {
                            if chapter_span.matches(&element) {
                                chapter_text.push_str("1 ");
                            } else if verse_sup.matches(&element) {
                                chapter_text.push_str("\n");
                                chapter_text.push_str(
                                    &element
                                        .text()
                                        .next()
                                        .unwrap()
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
    println!("{}", chapter_text);
}
