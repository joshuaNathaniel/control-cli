extern crate tree_sitter_typescript;
extern crate tree_sitter;

use tree_sitter::Language;

pub trait LanguagePlugin {
    fn language(&self) -> Language;
}

struct TreeSitterTypeScriptPlugin {
}

impl TreeSitterTypeScriptPlugin {
    fn new() -> TreeSitterTypeScriptPlugin {
        TreeSitterTypeScriptPlugin { }
    }
}

impl LanguagePlugin for TreeSitterTypeScriptPlugin {
    fn language(&self) -> Language { tree_sitter_typescript::language_tsx() }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn plugin_init() -> Box<dyn LanguagePlugin> {
    Box::new(TreeSitterTypeScriptPlugin::new())
}
