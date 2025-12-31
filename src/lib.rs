mod biblegateway;
use quick_xml::events::BytesStart;
use std::path::Path;
use std::{fs, io};

fn write_cachefile<P, C>(path: P, contents: C) -> io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    // Create the destination directories if they do not exist
    let dir = path.as_ref().parent().unwrap();
    fs::create_dir_all(dir)?;

    // Write the cache file
    fs::write(path, contents)
}

fn read_cachefile<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    fs::read_to_string(path)
}

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
