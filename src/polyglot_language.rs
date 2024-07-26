use tree_sitter::Node;
use crate::PolyglotZipper;
use crate::util::InvalidArgumentError;

pub struct Python {}

impl PolyLanguage for Python {
    fn get_child_index(&self) -> usize {
        0
    }

    fn get_node_kind(&self) -> &'static str {
        "call"
    }

    fn get_child_node_kind(&self) -> &'static str {
        "attribute"
    }

    fn get_lang_name(&self) -> &'static str {
        "python"
    }

    fn get_code_eval_arg(&self) -> Option<&str> {
        Some("string")
    }

    fn get_code_eval_file_arg(&self) -> Option<&str> {
        Some("path")
    }

    fn get_lang_arg(&self) -> Option<&str> {
        Some("language")
    }

    fn get_code_eval(&self) -> Option<&str> {
        None
    }

    fn get_code_eval_file(&self) -> Option<&str> {
        None
    }

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot.eval")
    }

    fn is_polyglot_import_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot.import_value")
    }

    fn is_polyglot_export_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot.export_value")
    }

    fn use_positional_args(&self) -> bool {
        false
    }

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)> {
        let arg1 = node.child(1)?.child(1)?.child(0)?;
        let arg2 = node.child(1)?.child(3)?.child(0)?;

        Some((arg1, arg2, None))
    }

    fn get_binding(&self, zipper: &PolyglotZipper) -> Option<String> {
        Some(String::from(zipper.child(1)?.child(1)?.code()))
    }

    fn get_treesitter_language(&self) -> Result<tree_sitter::Language, InvalidArgumentError> {
        Ok(tree_sitter_python::language())
    }
}
pub struct JavaScript {}

impl PolyLanguage for JavaScript {
    fn get_child_index(&self) -> usize {
        0
    }

    fn get_node_kind(&self) -> &'static str {
        "call_expression"
    }

    fn get_child_node_kind(&self) -> &'static str {
        "member_expression"
    }

    fn get_lang_name(&self) -> &'static str {
        "javascript"
    }

    fn get_code_eval_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval_file_arg(&self) -> Option<&str> {
        None
    }

    fn get_lang_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval(&self) -> Option<&str> {
        Some("eval")
    }

    fn get_code_eval_file(&self) -> Option<&str> {
        Some("evalFile")
    }

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool {
        matches!(node_code, "Polyglot.eval" | "Polyglot.evalFile")
    }

    fn is_polyglot_import_call(&self, node_code: &str) -> bool {
        matches!(node_code, "Polyglot.import")
    }

    fn is_polyglot_export_call(&self, node_code: &str) -> bool {
        matches!(node_code, "Polyglot.export")
    }

    fn use_positional_args(&self) -> bool {
        true
    }

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)> {
        let call_type = node.child(0)?.child(2)?; // function name
        let arg1 = node.child(1)?.child(1)?; // language
        let arg2 = node.child(1)?.child(3)?; // code

        Some((arg1, arg2, Some(call_type)))
    }

    fn get_binding(&self, _zipper: &PolyglotZipper) -> Option<String> {
        None
    }

    fn get_treesitter_language(&self) -> Result<tree_sitter::Language, InvalidArgumentError> {
        Ok(tree_sitter_javascript::language())
    }
}
pub struct Java {}

impl PolyLanguage for Java {
    fn get_child_index(&self) -> usize {
        2
    }

    fn get_node_kind(&self) -> &'static str {
        "method_invocation"
    }

    fn get_child_node_kind(&self) -> &'static str {
        "identifier"
    }

    fn get_lang_name(&self) -> &'static str {
        "java"
    }

    fn get_code_eval_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval_file_arg(&self) -> Option<&str> {
        None
    }

    fn get_lang_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval(&self) -> Option<&str> {
        Some("eval")
    }

    fn get_code_eval_file(&self) -> Option<&str> {
        None
    }

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool {
        matches!(node_code, "eval")
    }

    fn is_polyglot_import_call(&self, node_code: &str) -> bool {
        matches!(node_code, "getMember")
    }

    fn is_polyglot_export_call(&self, node_code: &str) -> bool {
        matches!(node_code, "putMember")
    }

    fn use_positional_args(&self) -> bool {
        true
    }

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)> {
        let arg1 = node.child(3)?.child(1)?; // language
        let arg2 = node.child(3)?.child(3)?; // code

        Some((arg1, arg2, None))
    }

    fn get_binding(&self, _zipper: &PolyglotZipper) -> Option<String> {
        None
    }

    fn get_treesitter_language(&self) -> Result<tree_sitter::Language, InvalidArgumentError> {
        Ok(tree_sitter_java::language())
    }
}
pub struct C {}

impl PolyLanguage for C {
    fn get_child_index(&self) -> usize {
        0
    }

    fn get_node_kind(&self) -> &'static str {
        "call_expression"
    }

    fn get_child_node_kind(&self) -> &'static str {
        "identifier"
    }

    fn get_lang_name(&self) -> &'static str {
        "c"
    }

    fn get_code_eval_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval_file_arg(&self) -> Option<&str> {
        None
    }

    fn get_lang_arg(&self) -> Option<&str> {
        None
    }

    fn get_code_eval(&self) -> Option<&str> {
        Some("polyglot_eval")
    }

    fn get_code_eval_file(&self) -> Option<&str> {
        Some("polyglot_eval_file")
    }

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot_eval" | "polyglot_eval_file")
    }

    fn is_polyglot_import_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot_import")
    }

    fn is_polyglot_export_call(&self, node_code: &str) -> bool {
        matches!(node_code, "polyglot_export")
    }

    fn use_positional_args(&self) -> bool {
        true
    }

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)> {
        let call_type = node.child(0)?;
        let arg1 = node.child(1)?.child(1)?; // language
        let arg2 = node.child(1)?.child(3)?; // code

        Some((arg1, arg2, Some(call_type)))
    }

    fn get_binding(&self, _zipper: &PolyglotZipper) -> Option<String> {
        None
    }

    fn get_treesitter_language(&self) -> Result<tree_sitter::Language, InvalidArgumentError> {
        Ok(tree_sitter_c::language())
    }
}

pub trait PolyLanguage {
    fn get_polyglot_call_lang<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        let child = node.child(self.get_child_index())?;

        if node.kind().eq(self.get_node_kind()) && child.kind().eq(self.get_child_node_kind()) {
            return Some(child);
        }
        None
    }

    fn get_child_index(&self) -> usize;
    fn get_node_kind(&self) -> &'static str;
    fn get_child_node_kind(&self) -> &'static str;

    fn get_lang_name(&self) -> &'static str;

    fn get_code_eval_arg(&self) -> Option<&str>;
    fn get_code_eval_file_arg(&self) -> Option<&str>;
    fn get_lang_arg(&self) -> Option<&str>;

    fn get_code_eval(&self) -> Option<&str>;
    fn get_code_eval_file(&self) -> Option<&str>;

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool;
    fn is_polyglot_import_call(&self, node_code: &str) -> bool;
    fn is_polyglot_export_call(&self, node_code: &str) -> bool;

    fn use_positional_args(&self) -> bool;

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)>;
    fn get_binding(&self, zipper: &PolyglotZipper) -> Option<String>;
    fn get_treesitter_language(&self) -> Result<tree_sitter::Language, InvalidArgumentError>;
}
