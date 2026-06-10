use libloading::{Library, Symbol};
use std::path::Path;
use tree_sitter::Language;

/// Loads a tree-sitter grammar from a compiled shared library.
/// The .so must export a symbol named `tree_sitter_<grammar_name>`.
///
/// # Safety
/// The library must be a valid tree-sitter grammar compiled for this platform.
pub fn load_grammar(grammar_dir: &Path, grammar_name: &str) -> Result<Language, String> {
    // e.g. grammar_name = "rust"  ->  symbol = "tree_sitter_rust"
    // ->  file   = "tree_sitter_rust.so" (linux)
    // = "tree_sitter_rust.dll" (windows)
    let lib_name = format!("tree_sitter_{}", grammar_name);
    let filename = platform_lib_name(&lib_name);
    let lib_path = grammar_dir.join(&filename);

    if !lib_path.exists() {
        return Err(format!(
            "Grammar library not found: {} (looked in {})",
            filename,
            grammar_dir.display()
        ));
    }

    // SAFETY: grammar .so files placed in the user's config dir are trusted
    let lib = unsafe {
        Library::new(&lib_path)
            .map_err(|e| format!("Failed to load {}: {}", lib_path.display(), e))?
    };

    // Leak the library so it lives for the program's lifetime
    // tree-sitter Language is just a thin pointer into the .so's static data.
    // Intentional. grammars loaded once, never unloaded.
    let lib = Box::leak(Box::new(lib));

    let symbol_name = format!("{}\0", lib_name); // null-terminated for libloading
    let language = unsafe {
        let func: Symbol<unsafe extern "C" fn() -> Language> = lib
            .get(symbol_name.as_bytes())
            .map_err(|e| format!("Symbol '{}' not found in {}: {}", lib_name, filename, e))?;
        func()
    };

    Ok(language)
}

pub fn platform_lib_name(stem: &str) -> String {
    #[cfg(target_os = "windows")]
    return format!("{}.dll", stem);
    #[cfg(target_os = "macos")]
    return format!("lib{}.dylib", stem);
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return format!("lib{}.so", stem);
}
