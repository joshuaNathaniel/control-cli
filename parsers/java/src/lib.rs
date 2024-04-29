extern crate tree_sitter_java;
extern crate tree_sitter;

use tree_sitter::Language;

pub trait LanguagePlugin {
    fn language(&self) -> Language;
}

struct TreeSitterJavaPlugin {
}

impl TreeSitterJavaPlugin {
    fn new() -> TreeSitterJavaPlugin {
        TreeSitterJavaPlugin { }
    }
}

impl LanguagePlugin for TreeSitterJavaPlugin {
    fn language(&self) -> Language {
        tree_sitter_java::language()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn plugin_init() -> Box<dyn LanguagePlugin> {
    Box::new(TreeSitterJavaPlugin::new())
}
