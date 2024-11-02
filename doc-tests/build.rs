//! Build script for `ferrunix`'s doc tests.
use skeptic::{generate_doc_tests, markdown_files_of_directory};

fn main() {
    // Add all markdown files in directory "book/".
    let mut mdbook_files = markdown_files_of_directory("../book/");
    // Also add "README.md" to the list of files.
    mdbook_files.push("../ferrunix/README.md".into());
    generate_doc_tests(&mdbook_files);
}
