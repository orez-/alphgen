pub struct SplitWhen<'a, T, F> {
    slice: &'a [T],
    f: F,
}

impl<'a, T, F> Iterator for SplitWhen<'a, T, F>
where F: FnMut(&T, &T) -> bool {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        let idx = self.slice
            .windows(2)
            .position(|w| (self.f)(&w[0], &w[1]))
            .unwrap_or(self.slice.len() - 1);
        let (start, end) = self.slice.split_at(idx + 1);
        self.slice = end;
        Some(start)
    }
}

/// Returns an iterator over subslices split when two consecutive elements
/// `a` and `b` match `f(&a, &b)`.
///
/// # Examples
///
/// ```ignore
/// let v = [1, 4, 5, 6, 5, 7, 8, 9];
/// let mut it = split_when(&v, |a, b| a + 1 == b);
///
/// assert_eq!(it.next().unwrap(), &[1]);
/// assert_eq!(it.next().unwrap(), &[4, 5, 6]);
/// assert_eq!(it.next().unwrap(), &[5]);
/// assert_eq!(it.next().unwrap(), &[7, 8, 9]);
/// assert!(it.next().is_none());
/// ```
pub fn split_when<T, F>(slice: &[T], f: F) -> SplitWhen<'_, T, F>
where F: FnMut(&T, &T) -> bool
{
    SplitWhen { slice, f }
}
