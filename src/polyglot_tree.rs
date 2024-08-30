use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use tree_sitter::{Node, Parser, Tree};

use crate::polyglot_language::PolyLanguage;

use super::util;

pub mod polyglot_processor;
pub mod polyglot_zipper;

/// An Abstract Syntax Tree (AST) spanning across multiple languages.
///
///
pub struct PolyglotTree {
    tree: Tree,
    code: String,
    working_dir: PathBuf,
    language: Box<dyn PolyLanguage>,
    node_to_subtrees_map: HashMap<usize, PolyglotTree>,
}

impl PolyglotTree {
    /// Given a program's code and a Language, returns a PolyglotTree instance that represents the program.
    ///
    /// The provided AST is built recursively from variations of the `polyglot.eval` function call in different languages, and can be traversed across language boundaries.
    ///
    /// Returns None if there was a problem during the parsing phase, which can happen either due to timeout or messing with the parser's cancellation flags.
    /// If you are not using tree-sitter in your program, you can safely assume this method will never return None;
    /// refer to the `tree_sitter::Parser::parse()` documentation for more information.
    ///
    /// # Arguments
    ///
    /// - `code` The code snippet to build the AST from, provided as any object that can be converted to a string.
    /// For proper use, ensure that `code.to_string()` would provide a syntactically correct code snippet.
    /// - `language` The Language variant that `code` is written in.
    ///
    /// # Examples
    ///
    /// ```
    /// use polyglot_ast::{polyglot_language, PolyglotTree};
    /// use polyglot_ast::polyglot_language::PolyLanguage;
    /// use polyglot_ast::util::Language;
    /// use polyglot_language::Python;
    ///
    /// let empty_tree: PolyglotTree = PolyglotTree::from("", Box::new(Python{})).expect("Python is a supported language");
    /// let tree: PolyglotTree = PolyglotTree::from("print(x*42)", Box::new(Python{})).expect("Python is a supported language");
    /// let py_js_tree: PolyglotTree = PolyglotTree::from("import polyglot\nprint(x*42)\npolyglot.eval(language=\"js\", string=\"console.log(42)\"", Box::new(Python{})).expect("Python is a supported language");
    /// ```
    ///
    /// # Panics
    ///
    /// This method can only panic if there is a problem while loading the language grammar into the parser, either in this call or subsequent recursive calls to build subtrees.
    /// This can only happen if tree_sitter and the grammars are of incompatible versions;
    /// either refer to the `tree_sitter::Parser::set_language()` documentation or directly contact polyglot_ast maintainers if this method keeps panicking.
    pub fn from(code: impl ToString, language: Box<dyn PolyLanguage>) -> Option<PolyglotTree> {
        let code = code.to_string();

        let mut parser = Parser::new();
        let ts_lang = util::language_struct_to_treesitter(&language).unwrap();

        parser
            .set_language(ts_lang)
            .expect("Error loading the language grammar into the parser; if this error persists, consider reporting it to the library maintainers.");

        let tree = parser.parse(code.as_str(), None)?;

        let mut result = PolyglotTree {
            tree,
            code,
            working_dir: PathBuf::new(),
            language,
            node_to_subtrees_map: HashMap::new(),
        };

        let mut map = HashMap::new();
        result.build_polyglot_tree(&mut map); // traverse the tree to build the subtrees
        result.node_to_subtrees_map = map; // set the map after its built
        Some(result)
    }

    /// Given a path to a file and a Language, returns a PolyglotTree instance that represents the program written in the file.
    ///
    /// The provided AST is built recursively from variations of the `polyglot.eval` function call in different languages,
    /// and can be traversed across language boundaries.
    ///
    /// Returns None if there was a problem while reading the file or during the parsing phase,
    /// which can happen either due to timeout or messing with the parser's cancellation flags;
    /// refer to the `tree_sitter::Parser::parse()` documentation for more information.
    ///
    /// If there is an error while reading the file, this method will "soft fail" and return None while printing a message to io::stderr.
    ///
    /// # Arguments
    ///
    /// - `path` A PathBuf to the file containing the code.
    /// - `language` The Language variant that the file at `path` is written in.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use polyglot_ast::{polyglot_language, PolyglotTree};
    /// use polyglot_ast::util::Language;
    /// use polyglot_language::Python;
    ///
    /// let file = PathBuf::from("TestSamples/export_x.py");
    /// let tree: PolyglotTree = PolyglotTree::from_path(file, Box::new(Python{})).expect("This test file exists");
    ///
    /// let file = PathBuf::from("this_file_does_not_exist.py");
    /// assert!(PolyglotTree::from_path(file, Box::new(Python{})).is_none());
    /// ```
    ///
    /// # Panics
    ///
    /// This method can only panic if there is a problem while loading the language grammar into the parser,
    /// either in this call or subsequent recursive calls to build subtrees.
    /// This can only happen if tree_sitter and the grammars are of incompatible versions;
    /// either refer to the `tree_sitter::Parser::set_language()` documentation or directly contact polyglot_ast maintainers if this method keeps panicking.
    pub fn from_path(path: PathBuf, language: Box<dyn PolyLanguage>) -> Option<PolyglotTree> {
        let file = path.clone();
        let code = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Warning: unable to create tree for file {} due to the following error: {e}",
                    file.to_str()?
                );
                return None;
            }
        };

        let mut parser = Parser::new();
        let ts_lang = util::language_struct_to_treesitter(&language).unwrap();

        parser
            .set_language(ts_lang)
            .expect("Error loading the language grammar into the parser; consider verifying your versions of the grammar and tree-sitter are compatible.");

        let tree = parser.parse(code.as_str(), None)?;

        let mut result = PolyglotTree {
            tree,
            code,
            working_dir: file.parent()?.to_path_buf(),
            language,
            node_to_subtrees_map: HashMap::new(),
        };

        let mut map = HashMap::new();
        result.build_polyglot_tree(&mut map);
        result.node_to_subtrees_map = map;
        Some(result)
    }

    /// Internal function to build a polyglot tree, which sets a specific working directory for the built subtree.
    /// This is used when a polyglot file has a polyglot call to raw code, to ensure any subsequent calls would properly locate files.
    ///
    /// # Arguments
    ///
    /// - `code` The code snippet to build the AST from, provided as any object that can be converted to a string.
    /// For proper use, ensure that `code.to_string()` would provide a syntactically correct code snippet.
    /// - `language` The Language variant that the file at `path` is written in.
    /// - `working_dir` a PathBuf of the parent directory of the file currently being processed.
    ///
    /// # Panics
    ///
    /// This method can only panic if there is a problem while loading the language grammar into the parser,
    /// either in this call or subsequent recursive calls to build subtrees.
    /// This can only happen if tree_sitter and the grammars are of incompatible versions;
    /// either refer to the `tree_sitter::Parser::set_language()` documentation or directly contact polyglot_ast maintainers if this method keeps panicking.
    fn from_directory(
        code: impl ToString,
        language: Box<dyn PolyLanguage>,
        working_dir: PathBuf,
    ) -> Option<PolyglotTree> {
        let code = code.to_string();

        let mut parser = Parser::new();
        let ts_lang = util::language_struct_to_treesitter(&language).unwrap();

        parser
            .set_language(ts_lang)
            .expect("Error loading the language grammar into the parser; consider verifying your versions of the grammar and tree-sitter are compatible.");

        let tree = parser.parse(code.as_str(), None)?;

        let mut result = PolyglotTree {
            tree,
            code,
            working_dir,
            language,
            node_to_subtrees_map: HashMap::new(),
        };

        let mut map = HashMap::new();
        result.build_polyglot_tree(&mut map);
        result.node_to_subtrees_map = map;
        Some(result)
    }

    /// Applies the given processor to the tree, starting from the root of the tree.
    /// For more information, refer to the PolyglotProcessor trait documentation.
    pub fn apply(&self, processor: &mut impl polyglot_processor::PolygotProcessor) {
        processor.process(polyglot_zipper::PolyglotZipper::from(self))
    }

    /// Internal function to get a node's source code.
    fn node_to_code(&self, node: Node) -> &str {
        &self.code[node.start_byte()..node.end_byte()]
    }

    /// Internal function to get the root node of the tree.
    fn root_node(&self) -> Node {
        self.tree.root_node()
    }

    /// Internal function to start building the polyglot mappings and subtrees.
    fn build_polyglot_tree(&self, node_tree_map: &mut HashMap<usize, PolyglotTree>) {
        let root = self.tree.root_node();
        self.build_polyglot_links(node_tree_map, root); // we get the root, and then call the recursive function
    }

    /// Internal recursive function that iterates over the nodes in the tree, and builds all subtrees as well as the polyglot link map.
    fn build_polyglot_links(&self, node_tree_map: &mut HashMap<usize, PolyglotTree>, node: Node) {
        if self.is_polyglot_eval_call(node) {
            if !self.make_subtree(node_tree_map, node).expect("Making the subtree failed") {
                // If building the subtree failed,
                // we want to soft fail (eg. not panic) to avoid interrupting the tree building.
                // Eventually, this should be made into a proper Error,
                // but for now for debugging purposes it just prints a warning.
                eprintln!(
                    "Warning: unable to make subtree for polyglot call at position {}",
                    node.start_position()
                )
            }
        } else {
            if let Some(child) = node.child(0) {
                self.build_polyglot_links(node_tree_map, child)
            };
            if let Some(sibling) = node.next_sibling() {
                self.build_polyglot_links(node_tree_map, sibling)
            };
        }
    }

    fn is_polyglot_eval_call(&self, node: Node) -> bool {
        match self.language.get_polyglot_call_lang(node) {
            None => false,
            Some(call_node) => self.language.is_polyglot_eval_call(self.node_to_code(call_node))
        }
    }

    fn is_polyglot_import_call(&self, node: Node) -> bool {
        match self.language.get_polyglot_call_lang(node) {
            None => false,
            Some(call_node) => self.language.is_polyglot_import_call(self.node_to_code(call_node))
        }
    }

    fn is_polyglot_export_call(&self, node: Node) -> bool {
        match self.language.get_polyglot_call_lang(node) {
            None => false,
            Some(call_node) => self.language.is_polyglot_export_call(self.node_to_code(call_node))
        }
    }

    fn make_subtree(&self, node_tree_map: &mut HashMap<usize, PolyglotTree>, node: Node) -> Option<bool> {
        let subtree: PolyglotTree;

        let args: (Node, Node, Option<Node>) = self.language.get_args(&node)?;

        let result: Option<PolyglotTree> = self.make_subtree_lang(args.0, args.1, args.2, self.language.use_positional_args());

        subtree = match result {
            Some(t) => t,
            None => return Some(false),
        };

        node_tree_map.insert(node.id(), subtree);

        Some(true) // signal everything went right
    }

    fn make_subtree_lang(&self, arg1: Node, arg2: Node, call_type: Option<Node>, positional: bool) -> Option<PolyglotTree> {
        if !positional {
            let mut new_code: Option<String> = None;
            let mut new_lang: Option<String> = None;
            let mut path: Option<PathBuf> = None;

            // Python polyglot calls use a single function and differentiate by argument names, which are mandatory.
            // We need to check both arguments for each possible case, and then check again at the end we have enough information.
            self.process_argument(arg1, &mut path, &mut new_lang, &mut new_code)?;
            self.process_argument(arg2, &mut path, &mut new_lang, &mut new_code)?;

            // We convert the language, if there was one
            let new_lang = match new_lang {
                Some(s) => match util::language_string_to_struct(s.as_str()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("Could not convert argument {s} to language due to error: {e}");
                        return None;
                    }
                },
                None => {
                    eprintln!(
                        "Warning: no language argument provided for polyglot call"
                    );
                    return None;
                }
            };

            let subtree = match new_code {
                Some(c) => Self::from_directory(c, new_lang, self.working_dir.clone())?,
                None => Self::from_path(
                    // No raw code, check for a path
                    match path {
                        Some(p) => p,
                        None => {
                            // No path either -> we cant build the tree
                            eprintln!("Warning:: no path or string argument provided to {{Lang}} polyglot call");
                            return None;
                        }
                    },
                    new_lang,
                )?,
            };

            return Some(subtree);
        }

        return match call_type {
            None => {
                // case where there are two args: lang then code

                match self.make_subtree_dir_positional_args(arg1, arg2) {
                    Ok(value) => value,
                    Err(value) => return value,
                }
            }
            Some(call_type) => {
                // JavaScript uses a different function for evaluating raw code and files, so we have two cases
                let eval_type = self.node_to_code(call_type);

                if eval_type == self.language.get_code_eval()? {
                    // Arguments are positional, and always at the same spot

                    match self.make_subtree_dir_positional_args(arg1, arg2) {
                        Ok(value) => value,
                        Err(value) => return value,
                    }
                } else if eval_type == self.language.get_code_eval_file()? {
                    match self.make_subtree_path_positional_args(arg1, arg2) {
                        Ok(value) => value,
                        Err(value) => return value,
                    }
                } else { None }
            }
        };
    }

    fn make_subtree_dir_positional_args(&self, arg1: Node, arg2: Node) -> Result<Option<PolyglotTree>, Option<PolyglotTree>> {
        let lang_s = util::strip_quotes(self.node_to_code(arg1));
        let new_code = util::strip_quotes(self.node_to_code(arg2));

        let new_lang = match util::language_string_to_struct(&lang_s) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Could not convert argument {lang_s} to language due to error: {e}", );
                return Err(None);
            }
        };

        Ok(Self::from_directory(new_code, new_lang, self.working_dir.clone()))
    }

    fn make_subtree_path_positional_args(&self, arg1: Node, arg2: Node) -> Result<Option<PolyglotTree>, Option<PolyglotTree>> {
        let lang_s = util::strip_quotes(self.node_to_code(arg1));

        let new_lang = match util::language_string_to_struct(&lang_s) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Could not convert argument {lang_s} to language due to error: {e}", );
                return Err(None);
            }
        };

        let tmp_path = util::strip_quotes(self.node_to_code(arg2));

        let mut path = self.working_dir.clone();

        let new_path = match PathBuf::from_str(tmp_path.as_str()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "Warning: could not build subtree for {} because of error {e}",
                    tmp_path.as_str()
                );
                return Err(None);
            }
        };

        path.push(new_path);

        Ok(Self::from_path(path, new_lang))
    }

    fn process_argument(&self, arg: Node, path: &mut Option<PathBuf>, new_lang: &mut Option<String>, new_code: &mut Option<String>) -> Option<()> {
        let value_node = arg.next_sibling()?.next_sibling()?;
        let tmp = util::strip_quotes(self.node_to_code(value_node));

        let arg_code = self.node_to_code(arg);

        if arg_code == self.language.get_code_eval_file_arg()? {
            *path = Some(self.working_dir.clone());
            let new_path = match PathBuf::from_str(tmp.as_str()) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "Warning: could not build subtree for {} because of error {e}",
                        tmp.as_str()
                    );
                    return None;
                }
            };
            *path = path.as_mut().map(|p| {
                p.push(new_path);
                p.clone()
            });
        }
        else if arg_code == self.language.get_lang_arg()? {
            *new_lang = Some(String::from(tmp.as_str()));
        }
        else if arg_code == self.language.get_code_eval_arg()? {
            *new_code = Some(String::from(tmp.as_str()));
        }
        else {
            eprintln!(
                "Warning: unable to handle polyglot call argument {arg_code} at position {}",
                arg.start_position()
            );
            return None;
        }

        Some(())
    }
}
