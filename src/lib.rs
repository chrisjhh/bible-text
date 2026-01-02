mod biblegateway;
use std::error::Error;

#[cfg(test)]
use std::{fs, io, path::Path};

pub trait GetChapterText {
    fn get_chapter_text(
        &self,
        book: usize,
        chapter: usize,
        version: &str,
    ) -> Result<Option<String>, Box<dyn Error>>;
}

#[cfg(test)]
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

#[cfg(test)]
fn read_cachefile<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    fs::read_to_string(path)
}
