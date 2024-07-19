use super::util;
use super::util::Language;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tree_sitter::{Node, Parser, Tree};

pub mod polyglot_processor;
pub mod polyglot_zipper;

/// An Abstract Syntax Tree (AST) spanning across multiple languages.
///
///
pub struct PolyglotTree {
    tree: Tree,
    code: String,
    working_dir: PathBuf,
    language: Language,
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
    /// use polyglot_ast::PolyglotTree;
    /// use polyglot_ast::util::Language;
    ///
    /// let empty_tree: PolyglotTree = PolyglotTree::from("", Language::Python).expect("Python is a supported language");
    /// let tree: PolyglotTree = PolyglotTree::from("print(x*42)", Language::Python).expect("Python is a supported language");
    /// let py_js_tree: PolyglotTree = PolyglotTree::from("import polyglot\nprint(x*42)\npolyglot.eval(language=\"js\", string=\"console.log(42)\"", Language::Python).expect("Python is a supported language");
    /// ```
    ///
    /// # Panics
    ///
    /// This method can only panic if there is a problem while loading the language grammar into the parser, either in this call or subsequent recursive calls to build subtrees.
    /// This can only happen if tree_sitter and the grammars are of incompatible versions;
    /// either refer to the `tree_sitter::Parser::set_language()` documentation or directly contact polyglot_ast maintainers if this method keeps panicking.
    pub fn from(code: impl ToString, language: Language) -> Option<PolyglotTree> {
        let code = code.to_string();

        let mut parser = Parser::new();
        let ts_lang = util::language_enum_to_treesitter(&language);

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
    /// use polyglot_ast::PolyglotTree;
    /// use polyglot_ast::util::Language;
    ///
    /// let file = PathBuf::from("TestSamples/export_x.py");
    /// let tree: PolyglotTree = PolyglotTree::from_path(file, Language::Python).expect("This test file exists");
    ///
    /// let file = PathBuf::from("this_file_does_not_exist.py");
    /// assert!(PolyglotTree::from_path(file, Language::Python).is_none());
    /// ```
    ///
    /// # Panics
    ///
    /// This method can only panic if there is a problem while loading the language grammar into the parser,
    /// either in this call or subsequent recursive calls to build subtrees.
    /// This can only happen if tree_sitter and the grammars are of incompatible versions;
    /// either refer to the `tree_sitter::Parser::set_language()` documentation or directly contact polyglot_ast maintainers if this method keeps panicking.
    pub fn from_path(path: PathBuf, language: Language) -> Option<PolyglotTree> {
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
        let ts_lang = util::language_enum_to_treesitter(&language);

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
        language: Language,
        working_dir: PathBuf,
    ) -> Option<PolyglotTree> {
        let code = code.to_string();

        let mut parser = Parser::new();
        let ts_lang = util::language_enum_to_treesitter(&language);

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
            if !self.make_subtree(node_tree_map, node) {
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

    fn get_polyglot_call_python(&self, node: Node) -> Option<&str> {
        let child = node.child(0)?;
        if node.kind().eq("call") && child.kind().eq("attribute") {
            return Some(self.node_to_code(child));
        }
        None
    }

    fn get_polyglot_call_js(&self, node: Node) -> Option<&str> {
        let child = node.child(0)?;
        if node.kind().eq("call_expression") && child.kind().eq("member_expression") {
            return Some(self.node_to_code(child));
        }
        None
    }

    fn get_polyglot_call_java(&self, node: Node) -> Option<&str> {
        let child = node.child(2)?;
        if node.kind().eq("method_invocation") && child.kind().eq("identifier") {
            return Some(self.node_to_code(child));
        }
        None
    }

    fn get_polyglot_call_c(&self, node: Node) -> Option<&str> {
        let child = node.child(0)?;
        if node.kind().eq("call_expression") && child.kind().eq("identifier") {
            return Some(self.node_to_code(child));
        }
        None
    }

    fn is_polyglot_eval_call(&self, node: Node) -> bool {
        match self.language {
            Language::Python => {
                matches!(self.get_polyglot_call_python(node), Some("polyglot.eval"))
            }
            Language::JavaScript => matches!(
                self.get_polyglot_call_js(node),
                Some("Polyglot.eval") | Some("Polyglot.evalFile")
            ),
            Language::Java => matches!(self.get_polyglot_call_java(node), Some("eval")),
            Language::C => matches!(self.get_polyglot_call_c(node), Some("polyglot_eval") | Some("polyglot_eval_file")),
        }
    }

    fn is_polyglot_import_call(&self, node: Node) -> bool {
        match self.language {
            Language::Python => matches!(
                self.get_polyglot_call_python(node),
                Some("polyglot.import_value")
            ),
            Language::JavaScript => {
                matches!(self.get_polyglot_call_js(node), Some("Polyglot.import"))
            }
            Language::Java => matches!(self.get_polyglot_call_java(node), Some("getMember")),
            Language::C => matches!(self.get_polyglot_call_c(node), Some("polyglot_import")),
        }
    }

    fn is_polyglot_export_call(&self, node: Node) -> bool {
        match self.language {
            Language::Python => matches!(
                self.get_polyglot_call_python(node),
                Some("polyglot.export_value")
            ),
            Language::JavaScript => {
                matches!(self.get_polyglot_call_js(node), Some("Polyglot.export"))
            }
            Language::Java => matches!(self.get_polyglot_call_java(node), Some("putMember")),
            Language::C => matches!(self.get_polyglot_call_c(node), Some("polyglot_export")),
        }
    }

    fn make_subtree(&self, node_tree_map: &mut HashMap<usize, PolyglotTree>, node: Node) -> bool {
        let subtree: PolyglotTree;
        let result: Option<PolyglotTree> = match self.language {
            // delegate to language specific subfunction
            Language::Python => self.make_subtree_python(&node),
            Language::JavaScript => self.make_subtree_js(&node),
            Language::Java => self.make_subtree_java(&node),
            Language::C => self.make_subtree_c(&node),
        };

        subtree = match result {
            Some(t) => t,
            None => return false,
        };

        node_tree_map.insert(node.id(), subtree);

        true // signal everything went right
    }

    fn make_subtree_lang(&self, arg1: Node, arg2: Node, call_type: Option<Node>, positional: bool) -> Option<PolyglotTree> {
        let code_eval = "eval";
        let code_eval_file = "evalFile";

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
                Some(s) => match util::language_string_to_enum(s.as_str()) {
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

                if eval_type == code_eval {
                    // Arguments are positional, and always at the same spot

                    match self.make_subtree_dir_positional_args(arg1, arg2) {
                        Ok(value) => value,
                        Err(value) => return value,
                    }
                } else if eval_type == code_eval_file {
                    match self.make_subtree_path_positional_args(arg1, arg2) {
                        Ok(value) => value,
                        Err(value) => return value,
                    }
                } else { None }
            }
        }

        None
    }

    fn make_subtree_dir_positional_args(&self, arg1: Node, arg2: Node) -> Result<Option<PolyglotTree>, Option<PolyglotTree>> {
        let lang_s = util::strip_quotes(self.node_to_code(arg1));
        let new_code = util::strip_quotes(self.node_to_code(arg2));

        let new_lang = match util::language_string_to_enum(&lang_s) {
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

        let new_lang = match util::language_string_to_enum(&lang_s) {
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

    fn make_subtree_python(&self, node: &Node) -> Option<PolyglotTree> {
        let arg1 = node.child(1)?.child(1)?.child(0)?;
        let arg2 = node.child(1)?.child(3)?.child(0)?;

        let mut new_code: Option<String> = None;
        let mut new_lang: Option<String> = None;
        let mut path: Option<PathBuf> = None;

        // Python polyglot calls use a single function and differentiate by argument names, which are mandatory.
        // We need to check both arguments for each possible case, and then check again at the end we have enough information.
        self.process_argument(arg1, &mut path, &mut new_lang, &mut new_code)?;
        self.process_argument(arg2, &mut path, &mut new_lang, &mut new_code)?;

        // We convert the language, if there was one
        let new_lang = match new_lang {
            Some(s) => match util::language_string_to_enum(s.as_str()) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Could not convert argument {s} to language due to error: {e}");
                    return None;
                }
            },
            None => {
                eprintln!(
                    "Warning: no language argument provided for polyglot call at position {}",
                    node.start_position()
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
                        eprintln!("Warning:: no path or string argument provided to Python polyglot call at position {}", node.start_position());
                        return None;
                    }
                },
                new_lang,
            )?,
        };
        Some(subtree)
    }

    fn process_argument(&self, arg: Node, path: &mut Option<PathBuf>, new_lang: &mut Option<String>, new_code: &mut Option<String>) -> Option<()> {
        let tmp = util::strip_quotes(self.node_to_code(arg.next_sibling()?.next_sibling()?));

        match self.node_to_code(arg) {
            "path" => {
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
            "language" => {
                *new_lang = Some(String::from(tmp.as_str()));
            }
            "string" => {
                *new_code = Some(String::from(tmp.as_str()));
            }
            other => {
                eprintln!(
                    "Warning: unable to handle polyglot call argument {other} at position {}",
                    arg.start_position()
                );
                return None;
            }
        }
        Some(())
    }

    fn make_subtree_js(&self, node: &Node) -> Option<PolyglotTree> {
        let call_type = node.child(0)?.child(2)?; // function name
        let arg1 = node.child(1)?.child(1)?; // language
        let arg2 = node.child(1)?.child(3)?; // code

        // JavaScript uses a different function for evaluating raw code and files, so we have two cases
        match self.node_to_code(call_type) {
            "eval" => {
                // Arguments are positional, and always at the same spot
                let tmp_lang = util::strip_quotes(self.node_to_code(arg1));
                let tmp_code = util::strip_quotes(self.node_to_code(arg2));

                let new_lang = match util::language_string_to_enum(tmp_lang.as_str()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!(
                            "Could not convert argument {} to language due to error: {e}",
                            tmp_lang.as_str()
                        );
                        return None;
                    }
                };

                let new_code = String::from(tmp_code.as_str());
                Self::from_directory(new_code, new_lang, self.working_dir.clone())
            }

            "evalFile" => {
                let tmp_lang = util::strip_quotes(self.node_to_code(arg1));

                let new_lang = match util::language_string_to_enum(tmp_lang.as_str()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!(
                            "Could not convert argument {} to language due to error: {e}",
                            tmp_lang.as_str()
                        );
                        return None;
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
                        return None;
                    }
                };

                path.push(new_path);

                Self::from_path(path, new_lang)
            }

            other => {
                eprintln!(
                    "Warning: unable to identify polyglot function call {other} at position {}",
                    node.start_position()
                );
                None
            }
        }
    }

    fn make_subtree_java(&self, node: &Node) -> Option<PolyglotTree> {
        // Java uses positional arguments, so they will always be accessible with the same route.
        let arg1 = node.child(3)?.child(1)?; // language
        let arg2 = node.child(3)?.child(3)?; // code

        let s = util::strip_quotes(self.node_to_code(arg1));

        let new_lang = match util::language_string_to_enum(&s) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Could not convert argument {s} to language due to error: {e}",);
                return None;
            }
        };

        let new_code = util::strip_quotes(self.node_to_code(arg2));
        Self::from_directory(new_code, new_lang, self.working_dir.clone())
    }

    fn make_subtree_c(&self, node: &Node) -> Option<PolyglotTree> {
        let call_type = node.child(0)?;

        // C also uses positional arguments
        let arg1 = node.child(1)?.child(1)?; // language
        let arg2 = node.child(1)?.child(3)?; // code

        match self.node_to_code(call_type) {
            "polyglot_eval" => {
                // Arguments are positional, and always at the same spot
                let tmp_lang = util::strip_quotes(self.node_to_code(arg1));
                let tmp_code = util::strip_quotes(self.node_to_code(arg2));

                let new_lang = match util::language_string_to_enum(tmp_lang.as_str()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!(
                            "Could not convert argument {} to language due to error: {e}",
                            tmp_lang.as_str()
                        );
                        return None;
                    }
                };

                let new_code = String::from(tmp_code.as_str());
                Self::from_directory(new_code, new_lang, self.working_dir.clone())
            }

            "polyglot_eval_file" => {
                let tmp_lang = util::strip_quotes(self.node_to_code(arg1));

                let new_lang = match util::language_string_to_enum(tmp_lang.as_str()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!(
                            "Could not convert argument {} to language due to error: {e}",
                            tmp_lang.as_str()
                        );
                        return None;
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
                        return None;
                    }
                };

                path.push(new_path);

                Self::from_path(path, new_lang)
            }

            other => {
                eprintln!(
                    "Warning: unable to identify polyglot function call {other} at position {}",
                    node.start_position()
                );
                None
            }
        }

    }
}
