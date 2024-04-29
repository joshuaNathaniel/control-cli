use crate::fs::{read_dir, read_file};
use crate::parser::{parse, traverse_and_select};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;
use tree_sitter::{Language, Node, TreeCursor};

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

impl Point {
    pub fn new(row: usize, column: usize) -> Point {
        Point { row, column }
    }
}

impl From<tree_sitter::Point> for Point {
    fn from(point: tree_sitter::Point) -> Point {
        Point::new(point.row, point.column)
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct CommentedCode {
    path: PathBuf,
    comment: String,
    content: String,
    start: Point,
    end: Point,
}

impl CommentedCode {
    pub fn new(
        path: PathBuf,
        comment: String,
        content: String,
        start: Point,
        end: Point,
    ) -> CommentedCode {
        CommentedCode {
            path,
            comment,
            content,
            start,
            end,
        }
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn get_comment(&self) -> String {
        self.comment.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn get_start(&self) -> Point {
        self.start.clone()
    }

    pub fn get_end(&self) -> Point {
        self.end.clone()
    }
}

pub fn get_control_commented_code(
    path: PathBuf,
    language: Language,
    ext: Vec<String>,
) -> Vec<CommentedCode> {
    let files = read_dir(path, &ext, read_file);
    let re = Regex::new(r"control(?:\s\w+)+").unwrap();
    let mut commented_code = Vec::new();
    for file in files {
        let tree = parse(&file.get_contents(), language);
        let nodes = traverse_and_select(tree.root_node(), |cursor: TreeCursor| -> Option<Node> {
            if cursor.node().kind() == "comment"
                || cursor.node().kind() == "block_comment"
                || cursor.node().kind() == "line_comment"
            {
                return Option::from(cursor.node());
            }
            return None;
        });

        for node in nodes {
            let contents = file.get_contents();
            let comment = node.utf8_text(contents.as_bytes()).unwrap();
            let next_sibling = node.next_sibling().unwrap();
            if re.is_match(comment) {
                commented_code.push(CommentedCode::new(
                    file.get_path(),
                    comment.to_string(),
                    next_sibling
                        .utf8_text(file.get_contents().as_bytes())
                        .unwrap()
                        .to_string(),
                    Point::from(node.next_sibling().unwrap().start_position()),
                    Point::from(node.next_sibling().unwrap().end_position()),
                ));
            }
        }
    }
    return commented_code;
}

pub fn get_common_values<T: Eq + Clone + Debug>(v1: &Vec<T>, v2: &Vec<T>) -> Vec<T> {
    let mut common_values = Vec::new();
    for value in v1 {
        if v2.contains(&value) {
            common_values.push(value.clone());
        }
    }
    return common_values;
}

#[test]
fn check_get_control_commented_code() {
    use crate::parser::SupportedLanguage;
    let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = workspace_dir.join("tests/resources/js");
    let ext = vec!["js".to_string()];
    let language = SupportedLanguage::JavaScript.language();
    let commented_code =
        get_control_commented_code(path, language, ext);
    assert_eq!(commented_code.len(), 2);
    assert!(commented_code[0].get_path().to_str().unwrap().contains("tests/resources/js/subdirectory/submodule.js"));
    assert_eq!(commented_code[0].get_comment(), "// control SUB-1");
    assert_eq!(
        commented_code[0].get_content(),
        "const submodule = () => {\n  return 'submodule';\n}"
    );
    assert!(commented_code[1].get_path().to_str().unwrap().contains("tests/resources/js/index.js"));
    assert_eq!(
        commented_code[1].get_comment(),
        "/* control HE-110 JS-1 */"
    );
    assert_eq!(
        commented_code[1].get_content(),
        "console.log('Hello world!');"
    );
}
