use tree_sitter::Node;

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

    fn is_polyglot_eval_call(&self, node_code: &str) -> bool;
    fn is_polyglot_import_call(&self, node_code: &str) -> bool;
    fn is_polyglot_export_call(&self, node_code: &str) -> bool;

    fn use_positional_args(&self) -> bool;

    fn get_args<'a>(&self, node: &'a Node) -> Option<(Node<'a>, Node<'a>, Option<Node<'a>>)>;
}
