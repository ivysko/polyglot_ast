use std::path::PathBuf;

use polyglot_ast::{PolyglotTree, TreePrinter};
use polyglot_ast::polyglot_language::{Java};
// use polyglot_ast::polyglot_language::{C, Java, Python};

fn main() {
    /*let file = PathBuf::from("TestSamples/export_x.py");
    let tree =
        PolyglotTree::from_path(file, Box::new(Python{})).expect("Should not have parsing issues");*/

    let file = PathBuf::from("TestSamples/JavaTest.java");
    let tree =
        PolyglotTree::from_path(file, Box::new(Java{})).expect("Should not have parsing issues");

    /*let file = PathBuf::from("TestSamples/export_x.c");
    let tree =
        PolyglotTree::from_path(file, Box::new(C{})).expect("Should not have parsing issues");*/
    let mut tp = TreePrinter::new();
    tree.apply(&mut tp);
    println!("{}", tp.get_result())
}
