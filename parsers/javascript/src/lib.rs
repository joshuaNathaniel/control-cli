extern crate tree_sitter_javascript;
extern crate tree_sitter;

use tree_sitter::Language;

pub trait LanguagePlugin {
    fn language(&self) -> Language;
}

struct TreeSitterJavaScriptPlugin {
}

impl TreeSitterJavaScriptPlugin {
    fn new() -> TreeSitterJavaScriptPlugin {
        TreeSitterJavaScriptPlugin { }
    }
}

impl LanguagePlugin for TreeSitterJavaScriptPlugin {
    fn language(&self) -> Language {
        tree_sitter_javascript::language()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn plugin_init() -> Box<dyn LanguagePlugin> {
    Box::new(TreeSitterJavaScriptPlugin::new())
}
