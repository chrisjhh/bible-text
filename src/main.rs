use std::borrow::Cow;

use reqwest;
use quick_xml::{Reader};
use quick_xml::events::Event;

fn main() {
    let text = reqwest::blocking::get("https://www.biblegateway.com/passage/?search=ge%201&version=NIV")
        .unwrap()
        .text()
        .unwrap();
    //println!("Response Text: {}", text);

    // Read block <dic class="passage-text">
    // Split on <sup class="versenum">
    let mut reader = Reader::from_str(&text);
    reader.config_mut().trim_text(true);

    let mut text: String = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"div" => {
                for attr in e.attributes() {
                    let attr = attr.unwrap();
                    if attr.key.as_ref() == b"class" && attr.unescape_value().unwrap() == "passage-text" {
                        // Found passage-text div
                        let mut depth = 0;
                        loop {
                            match reader.read_event() {
                                Ok(Event::Start(ref e)) if e.name().as_ref() == b"sup" => {
                                    let mut sup_text = Cow::Borrowed("");
                                    if let Ok(Event::Text(e)) = reader.read_event() {
                                        sup_text= e.decode().unwrap().to_owned();
                                    }
                                    for attr in e.attributes() {
                                        let attr = attr.unwrap();
                                        if attr.key.as_ref() == b"class" && attr.unescape_value().unwrap() == "versenum" {
                                            // Found versenum sup
                                                text.push_str("\n");
                                                text.push_str(&sup_text);
                                        }
                                    }
                                }
                                Ok(Event::Text(e)) => {
                                    let verse_text = e.decode().unwrap();
                                    if !verse_text.is_empty() {
                                        // println!("Verse Text: {}", verse_text);
                                        text.push_str(" ");
                                        text.push_str(&verse_text);
                                    }
                                }
                                Ok(Event::Start(ref e)) if e.name().as_ref() == b"div" => depth += 1,
                                Ok(Event::End(ref e)) if e.name().as_ref() == b"div" => {
                                    if depth == 0 {
                                        break;
                                    }
                                    depth -= 1;},
                                Ok(Event::Eof) => break,
                                //Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                                _ => (),
                            }
                        }
                    }
                }
            },
            Ok(Event::Eof) => break,
            //Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    println!("{}", text);
}
