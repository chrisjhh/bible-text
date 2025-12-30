use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use reqwest;

fn has_attr(e: &BytesStart, key: &[u8], value: &str) -> bool {
    let mut found: bool = false;
    for attr in e.attributes() {
        let attr = attr.unwrap();
        if attr.key.as_ref() == key
            && attr
                .unescape_value()
                .unwrap()
                .split(" ")
                .any(|x| x == value)
        {
            found = true;
            break;
        }
    }
    found
}

fn main() {
    let text =
        reqwest::blocking::get("https://www.biblegateway.com/passage/?search=ps%20144&version=NIV")
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
                                        text.push_str(&e.decode().unwrap());
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
    println!("{}", text);
}
/*
*


if e.name().as_ref() == b"div" => {
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
*/
