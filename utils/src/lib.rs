use std::ops::DerefMut;

use nucleo::pattern::{Atom, AtomKind, CaseMatching};
use nucleo::Config;
use parking_lot::Mutex;

struct LazyMutex<T> {
    inner: Mutex<Option<T>>,
    init: fn() -> T,
}

impl<T> LazyMutex<T> {
    const fn new(init: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(None),
            init,
        }
    }

    fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        parking_lot::MutexGuard::map(self.inner.lock(), |val| val.get_or_insert_with(self.init))
    }
}

static MATCHER: LazyMutex<nucleo::Matcher> = LazyMutex::new(nucleo::Matcher::default);

pub fn fuzzy_match<T: AsRef<str>>(
    pattern: &str,
    items: impl IntoIterator<Item = T>,
) -> Vec<(T, u16)> {
    let mut matcher = MATCHER.lock();
    matcher.config = Config::DEFAULT;
    let pattern = Atom::new(pattern, CaseMatching::Smart, AtomKind::Fuzzy, false);
    pattern.match_list(items, &mut matcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        let items = vec!["foo/bar", "bar/foo", "foobar"];
        let result = fuzzy_match("foo", items);
        assert_eq!(
            result,
            vec![("foo/bar", 88), ("foobar", 88), ("bar/foo", 84)]
        );
    }
}
