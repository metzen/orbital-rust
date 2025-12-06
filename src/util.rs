// TODO: Replace this with std::cmp::minmax_by_key (https://github.com/rust-lang/rust/issues/115939).
/// Returns minimum and maximum values with respect to the specified key function.
///
/// Returns [v1, v2] if the comparison determines them to be equal.
pub fn minmax_by_key<T, F, K>(v1: T, v2: T, mut f: F) -> [T; 2]
where
    F: FnMut(&T) -> K,
    K: PartialOrd,
{
    if f(&v2) < f(&v1) { [v2, v1] } else { [v1, v2] }
}

#[cfg(test)]
#[path = "util_test.rs"]
mod util_test;
