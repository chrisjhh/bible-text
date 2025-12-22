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
                                    for attr in e.attributes() {
                                        let attr = attr.unwrap();
                                        if attr.key.as_ref() == b"class" && attr.unescape_value().unwrap() == "versenum" {
                                            // Found versenum sup
                                            if let Ok(Event::Text(e)) = reader.read_event() {
                                                let verse_num = e.decode().unwrap().to_owned();
                                                println!("Verse Number: {}", verse_num);
                                            }
                                        }
                                    }
                                }
                                Ok(Event::Text(e)) => {
                                    let verse_text = e.decode().unwrap().to_owned();
                                    if !verse_text.trim().is_empty() {
                                        println!("Verse Text: {}", verse_text);
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

}
