pub mod env;
pub mod error;
pub mod infer;
pub mod ty;

use crate::config::CompilerOptions;
use crate::type_checker::error::TypeError;
use crate::type_checker::ty::Type;

#[derive(Default)]
pub struct TypeChecker {
  pub errors: Vec<TypeError>,
  pub current_class_type: Option<Type>,
  /// Expected return type for the current function body (unwrapped T from Promise<T> for async).
  pub current_return_type: Option<Type>,
  pub narrowed: std::collections::HashMap<String, Type>,
  /// Module path → exported symbol name → type. Used for cross-file type resolution.
  pub module_registry: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
  /// Path of the file currently being type-checked (for resolving relative imports).
  pub current_module: std::path::PathBuf,
  /// Compiler options from tsconfig.json.
  pub options: CompilerOptions,
  /// Project root directory (for resolving path aliases).
  pub root_dir: std::path::PathBuf,
}

impl TypeChecker {
  #[must_use]
  pub fn new() -> Self {
    Self {
      errors: Vec::new(),
      current_class_type: None,
      current_return_type: None,
      narrowed: std::collections::HashMap::new(),
      module_registry: std::collections::HashMap::new(),
      current_module: std::path::PathBuf::new(),
      options: CompilerOptions::default(),
      root_dir: std::path::PathBuf::new(),
    }
  }

  /// Check if strict mode is enabled.
  #[must_use]
  pub fn is_strict(&self) -> bool {
    self.options.strict.unwrap_or(false)
  }
}
