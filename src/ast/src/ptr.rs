use std::sync::Arc;

/// P<T> is alias type for sharable Mu AST components (such as Value, MuFuncSig, MuType, etc)
// This design is similar to P<T> in rustc compiler.
// However, instead of using Box<T>, we use Arc<T> to encourage sharing
pub type P<T> = Arc<T>;

#[allow(non_snake_case)]
/// Construct a `P<T>` from a `T` value.
pub fn P<T>(value: T) -> P<T> {
    Arc::new(value)
}