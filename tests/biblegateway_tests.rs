use bible_text::BibleGateway;
use bible_text::GetChapterText;
use biblearchive::BARFile;
use difference::Changeset;
use rand::Rng;
use std::env;
use std::fs;
use std::path::PathBuf;

#[ignore = "Uses later version of NIV so passages do not match"]
#[test]
fn test_biblegateway_niv() {
    // Get the existing BAR file to the NIV
    let data_dir = env::var("BAR_DATADIR");
    assert!(
        data_dir.is_ok(),
        "Environment variable BAR_DATADIR must be set for this test."
    );
    let mut path = PathBuf::new();
    path.push(data_dir.unwrap());

    // Get the NIV v1 barfile
    path.push("niv_v1.bar");
    assert!(
        fs::exists(&path).unwrap(),
        "Cannot find required niv_v1.bar file in datadir"
    );

    let bar = BARFile::open(&path).unwrap();

    // Test 10 randomly chosen chapters
    let mut rng = rand::rng();
    for _ in 0..10 {
        let book_number = rng.random_range(1..=66);
        let book = bar.book(book_number).unwrap();
        let num_chapters = book.number_of_chapters();
        let chapter_number = rng.random_range(1..=num_chapters);
        let chapt = book.chapter(chapter_number).unwrap();
        let mut chapter_text = String::new();
        for (i, verse) in chapt.enumerated_verses() {
            if !chapter_text.is_empty() {
                chapter_text.push_str("\n");
            }
            chapter_text.push_str(&format!("{} {}", i, &verse));
        }
        let bg = BibleGateway;
        let mut fetched_text = bg
            .get_chapter_text(book_number as usize, chapter_number as usize, "NIVUK")
            .unwrap()
            .unwrap();

        // Do some replacements to avoid false positives
        fetched_text = fetched_text.replace("’", "'");
        fetched_text = fetched_text.replace("—", "--");
        fetched_text = fetched_text.replace("“", "\"");
        fetched_text = fetched_text.replace("”", "\"");

        // Compare the two
        let changeset = Changeset::new(&chapter_text, &fetched_text, " ");
        assert!(
            chapter_text == fetched_text,
            "{} {}\n{}",
            book.book_abbrev(),
            chapter_number,
            changeset
        );
    }
}

#[ignore = "Some whitespace differences to resolve."]
#[test]
fn test_biblegateway_esv() {
    // Get the existing BAR file to the NIV
    let data_dir = env::var("BAR_DATADIR");
    assert!(
        data_dir.is_ok(),
        "Environment variable BAR_DATADIR must be set for this test."
    );
    let mut path = PathBuf::new();
    path.push(data_dir.unwrap());

    // Get the NIV v1 barfile
    path.push("esv.ibar");
    assert!(
        fs::exists(&path).unwrap(),
        "Cannot find required esv.ibar file in datadir"
    );

    let bar = BARFile::open(&path).unwrap();

    // Test 10 randomly chosen chapters
    let mut rng = rand::rng();
    for _ in 0..10 {
        let book_number = rng.random_range(1..=66);
        let book = bar.book(book_number).unwrap();
        let num_chapters = book.number_of_chapters();
        let chapter_number = rng.random_range(1..=num_chapters);
        let chapt = book.chapter(chapter_number).unwrap();
        let mut chapter_text = String::new();
        for (i, verse) in chapt.enumerated_verses() {
            if !chapter_text.is_empty() {
                chapter_text.push_str("\n");
            }
            chapter_text.push_str(&format!("{} {}", i, &verse));
        }
        let bg = BibleGateway;
        let mut fetched_text = bg
            .get_chapter_text(book_number as usize, chapter_number as usize, "ESV")
            .unwrap()
            .unwrap();

        // Do some replacements to avoid false positives
        //fetched_text = fetched_text.replace("LORD", "Lord");
        //fetched_text = fetched_text.replace("—", "--");
        //fetched_text = fetched_text.replace("“", "\"");
        //fetched_text = fetched_text.replace("”", "\"");

        // Compare the two
        let changeset = Changeset::new(&chapter_text, &fetched_text, " ");
        assert!(
            chapter_text == fetched_text,
            "{} {}\n{}",
            book.book_abbrev(),
            chapter_number,
            changeset
        );
    }
}
