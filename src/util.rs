use thiserror::Error;

use crate::polyglot_language::{C, Java, JavaScript, PolyLanguage, Python};

#[derive(Error, Debug)]
#[error("Invalid argument received")]
pub struct InvalidArgumentError;

/// An enumeration that represents all languages supported by this crate. Current options are Python, JavaScript and Java.
pub enum Language {
    Python,
    JavaScript,
    /// Warning: Java language support is very partial and limited to string literal usage. Keep this in mind when writing your programs
    Java,
    C,
}

/// Returns a String identical to the provided slice but with leading and trailing characters removed.
/// In practice, this is mostly used to remove quotes from string literals, but the function does not actually check which characters it removes.
///
/// # Examples
/// ```
/// use polyglot_ast::util;
///
/// let s = "\'Hello!\'";
/// let stripped = util::strip_quotes(&s);
/// assert_eq!(stripped, String::from("Hello!"));
///
/// let stripped_again = util::strip_quotes(stripped.as_str());
/// assert_eq!(stripped_again, String::from("ello"));
///
/// ```
pub fn strip_quotes(s: &str) -> String {
    let mut tmp = s.chars();
    tmp.next();
    tmp.next_back();
    String::from(tmp.as_str())
}

/// Returns the treesitter language corresponding to the string slice passed.
///
/// If the string slice does not match any supported language, the return value will be an InvalidArgumentError.
///
/// # Examples
/// Valid use-case:
/// ```
/// use polyglot_ast::util;
///
/// let language = util::language_string_to_treesitter("python").expect("Python is a supported polyglot AST language");
///
/// assert_eq!(language, tree_sitter_python::language());
/// ```
/// Invalid use-case:
/// ```
/// use polyglot_ast::util;
/// use util::InvalidArgumentError;
///
/// let language = util::language_string_to_treesitter("go");
/// let invalid: InvalidArgumentError = match language {
///     Ok(_) => panic!("Go is not a supported language"),
///     Err(e) => e,
/// };
/// ```
pub fn language_string_to_treesitter(
    lang: &str,
) -> Result<tree_sitter::Language, InvalidArgumentError> {
    language_struct_to_treesitter(&language_string_to_struct(lang)?)
}

/// Returns the treesitter language corresponding to the Language enum reference passed.
///
/// # Example
/// ```
/// use polyglot_ast::util;
/// use polyglot_ast::util::InvalidArgumentError;
/// use polyglot_ast::polyglot_language::{C, PolyLanguage};
/// use util::Language;
///
/// let c: Box<(dyn PolyLanguage)> = Box::new(C{});
///
/// let language: Result<tree_sitter::Language, InvalidArgumentError> = util::language_struct_to_treesitter(&c);
///
/// assert_eq!(language.is_ok(), true);
/// assert_eq!(language.unwrap(), tree_sitter_c::language());
/// ```
pub fn language_struct_to_treesitter(lang: &Box<dyn PolyLanguage>) -> Result<tree_sitter::Language, InvalidArgumentError> {
    match lang.get_lang_name() {
        "python" => Ok(tree_sitter_python::language()),
        "javascript" => Ok(tree_sitter_javascript::language()),
        "java" => Ok(tree_sitter_java::language()),
        "c" => Ok(tree_sitter_c::language()),

        _ => Err(InvalidArgumentError)
    }
}

/// Returns the Language enum corresponding to the passed string slice
/// If the string slice does not match any supported language, the return value will be an InvalidArgumentError.
/// # Examples
/// Valid use-case:
/// ```
/// use polyglot_ast::util;
/// use polyglot_ast::polyglot_language::{PolyLanguage, Python};
/// use util::Language;
///
/// let language: Box<dyn PolyLanguage> = util::language_string_to_struct("python").expect("Python is a supported polyglot AST language");
/// let python = Box::new(Python{});
///
/// assert!(matches!(language, python));
/// ```
/// Invalid use-case:
/// ```
/// use polyglot_ast::util;
/// use util::InvalidArgumentError;
///
/// let language = util::language_string_to_treesitter("go");
/// let invalid: InvalidArgumentError = match language {
///     Ok(_) => panic!("Go is not a supported language"),
///     Err(e) => e,
/// };
/// ```
pub fn language_string_to_struct(lang: &str) -> Result<Box<dyn PolyLanguage>, InvalidArgumentError> {
    match lang {
        "python" => Ok(Box::new(Python{})),
        "js" | "javascript" => Ok(Box::new(JavaScript{})),
        "java" => Ok(Box::new(Java{})),
        "c" => Ok(Box::new(C{})),
        _ => Err(InvalidArgumentError),
    }
}
