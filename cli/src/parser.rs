use libloading::{Library, Symbol};
use std::{env, io};
use std::process::exit;
use tree_sitter::{Language, Node, Parser, Tree, TreeCursor};
#[cfg(debug_assertions)]
use std::path::PathBuf;

trait LanguagePlugin {
    fn language(&self) -> Language;
}

pub enum SupportedLanguage {
    Java,
    JavaScript,
    TypeScript,
}

impl From<String> for SupportedLanguage {
    fn from(language: String) -> SupportedLanguage {
        match language.as_str() {
            "java" => SupportedLanguage::Java,
            "js" => SupportedLanguage::JavaScript,
            "ts" => SupportedLanguage::TypeScript,
            _ => {
                println!("Error: Unsupported language");
                exit(1);
            }
        }
    }
}

impl SupportedLanguage {
    fn load_plugin(name: &str) -> Result<Box<dyn LanguagePlugin>, io::Error> {
        unsafe {
            let lib_name = format!("{}.{}", name, env::consts::DLL_EXTENSION);
            let file_path = {
                #[cfg(debug_assertions)]
                {
                    let file_path = match PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent() {
                        Some(path) => path.join("target").join("debug").join(lib_name),
                        None => {
                            println!("Error: Could not get parent directory of config file");
                            exit(1);
                        }
                    };

                    file_path
                }
                #[cfg(not(debug_assertions))]
                {
                    let file_path = match confy::get_configuration_file_path("control", "config") {
                        Ok(path) => match path.parent() {
                            Some(path) => path.join(format!("{}", lib_name)),
                            None => {
                                println!("Error: Could not get parent directory of config file");
                                exit(1);
                            }
                        },
                        Err(err) => {
                            println!("Error: {}", err);
                            exit(1);
                        }
                    };
                    file_path
                }
            };

            #[cfg(target_os = "linux")]
                let lib: Library = {
                match ::libloading::os::unix::Library::open(Some(file_path), 0x2 | 0x1000) {
                    Ok(lib) => lib,
                    _ => {
                        println!("Error loading plugin in linux");
                        exit(1);
                    }
                }.into()
            };
            #[cfg(not(target_os = "linux"))]
                let lib = match Library::new(file_path) {
                Ok(lib) => lib,
                Err(e) => {
                    println!("Error loading plugin: {}", e);
                    exit(1);
                }
            };
            let func: Symbol<fn() -> Box<dyn LanguagePlugin>> = match lib.get(b"plugin_init\0") {
                Ok(f) => f,
                Err(e) => {
                    println!("Error initializing plugin: {}", e);
                    exit(1);
                }
            };
            Ok(func())
        }
    }

    #[allow(dead_code)]
    pub fn language(&self) -> Language {
        match self {
            SupportedLanguage::Java => match SupportedLanguage::load_plugin("libparser_java") {
                Ok(plugin) => {
                    plugin.language()
                }
                Err(_) => {
                    println!("Error: Could not load plugin at language: Java");
                    exit(1);
                }
            }
            SupportedLanguage::JavaScript => match SupportedLanguage::load_plugin("libparser_js") {
                Ok(plugin) => {
                    plugin.language()
                }
                Err(_) => {
                    println!("Error: Could not load plugin at language: JavaScript");
                    exit(1);
                }
            }
            SupportedLanguage::TypeScript => match SupportedLanguage::load_plugin("libparser_ts") {
                Ok(plugin) => {
                    plugin.language()
                }
                Err(_) => {
                    println!("Error: Could not load plugin at language: TypeScript");
                    exit(1);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn parse(code: &str, language: Language) -> Tree {
    let mut parser = Parser::new();
    match parser.set_language(language) {
        Ok(_) => {}
        Err(err) => {
            println!("Error: {}", err);
        }
    }

    return parser.parse(code, None).unwrap();
}

pub fn traverse_and_select(node: Node, select: fn(TreeCursor) -> Option<Node>) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut cursor = node.walk();
    cursor.goto_first_child();
    loop {
        let selected_node = select(cursor.clone());
        match selected_node {
            Some(node) => nodes.push(node),
            None => (),
        }
        if cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return nodes;
            }
        }
    }
}

#[test]
fn check_tree_traversal() {
    let code = r#"// control T84
public class Test {
    public static void main(String[] args) {
        // control T83
        System.out.println("Hello, World!");
    }
}"#;
    let tree = parse(code, SupportedLanguage::Java.language());
    let nodes = traverse_and_select(tree.root_node(), |mut cursor: TreeCursor| -> Option<Node> {
        if cursor.node().kind() == "comment"
            || cursor.node().kind() == "block_comment"
            || cursor.node().kind() == "line_comment"
        {
            cursor.goto_next_sibling();
            return Option::from(cursor.node());
        }

        return None;
    });
    assert_eq!(nodes.len(), 2);

    let node_t84 = nodes[0];
    assert_eq!(
        node_t84.utf8_text(code.as_bytes()).unwrap(),
        r#"public class Test {
    public static void main(String[] args) {
        // control T83
        System.out.println("Hello, World!");
    }
}"#
    );
    let node_t83 = nodes[1];
    assert_eq!(
        node_t83.utf8_text(code.as_bytes()).unwrap(),
        "System.out.println(\"Hello, World!\");"
    );
}

#[test]
fn check_parse_javascript_comment() {
    let code = "function add(a, b) {
  // control T84
  return a + b;
}";
    let tree = parse(code, SupportedLanguage::JavaScript.language());
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling();
    cursor.goto_first_child();
    cursor.goto_next_sibling();

    assert_eq!(cursor.node().kind(), "comment");
    assert_eq!(
        cursor.node().utf8_text(code.as_bytes()).unwrap(),
        "// control T84"
    );
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "return_statement");
}

#[test]
fn check_parse_javascript_multiline_comment() {
    let code = "function add(a, b) {
  /**
   * control T84
  **/
  return a + b;
}";
    let tree = parse(code, SupportedLanguage::JavaScript.language());
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling();
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "comment");
    assert_eq!(
        cursor.node().utf8_text(code.as_bytes()).unwrap(),
        "/**\n   * control T84\n  **/"
    );
}

#[test]
fn check_parse_javascript() {
    let code = "function add(a, b) {
  return a + b;
}";
    let tree = parse(code, SupportedLanguage::JavaScript.language());
    let root_node = tree.root_node();

    assert_eq!(root_node.kind(), "program");

    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "function_declaration");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "function");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(cursor.node().utf8_text(code.as_bytes()).unwrap(), "add");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "formal_parameters");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "statement_block");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "{");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "return_statement");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "return");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "binary_expression");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "identifier");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "+");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), ";");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "}");
}

#[test]
fn check_parse_java_comment() {
    let code = "public class Math {
    // control T84
    public static int add(int a, int b) {
        return a + b;
    }
}";
    let tree = parse(code, SupportedLanguage::Java.language());
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "class_declaration");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "modifiers");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(cursor.node().utf8_text(code.as_bytes()).unwrap(), "Math");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class_body");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "{");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "line_comment");
    assert_eq!(
        cursor.node().utf8_text(code.as_bytes()).unwrap(),
        "// control T84"
    );
}

#[test]
fn check_parse_java_multiline_comment() {
    let code = "public class Math {
    /*
      control T84
    */
    public static int add(int a, int b) {
        return a + b;
    }
}";
    let tree = parse(code, SupportedLanguage::Java.language());
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "class_declaration");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "modifiers");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(cursor.node().utf8_text(code.as_bytes()).unwrap(), "Math");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class_body");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "{");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "block_comment");
    assert_eq!(
        cursor.node().utf8_text(code.as_bytes()).unwrap(),
        "/*\n      control T84\n    */"
    );
}

#[test]
fn check_parse_java() {
    let code = "public class Math {
    public static int add(int a, int b) {
        return a + b;
    }
}";
    let tree = parse(code, SupportedLanguage::Java.language());
    let root_node = tree.root_node();

    assert_eq!(root_node.kind(), "program");
    assert_eq!(root_node.start_position().row, 0);
    assert_eq!(root_node.start_position().column, 0);
    assert_eq!(root_node.end_position().row, 4);
    assert_eq!(root_node.end_position().column, 1);

    let mut cursor = root_node.walk();

    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "class_declaration");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "modifiers");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(cursor.node().utf8_text(code.as_bytes()).unwrap(), "Math");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "class_body");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "{");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "method_declaration");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "modifiers");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "integral_type");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    assert_eq!(cursor.node().utf8_text(code.as_bytes()).unwrap(), "add");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "formal_parameters");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "block");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "{");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "return_statement");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "return");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "binary_expression");
    cursor.goto_first_child();
    assert_eq!(cursor.node().kind(), "identifier");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "+");
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "identifier");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), ";");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "}");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "block");
    cursor.goto_parent();
    cursor.goto_next_sibling();
    assert_eq!(cursor.node().kind(), "}");
}
