#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) struct BSearch {
    pub len: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
}

impl BSearch {
    pub(crate) fn from(len: u16, record_size: u16) -> Self {
        let search_range = prev_power_of_two(len);
        let entry_selector = search_range.ilog2() as u16;
        let range_shift = len - search_range;
        BSearch {
            len: len * record_size,
            search_range: search_range * record_size,
            entry_selector,
            range_shift: range_shift * record_size,
        }
    }
}

/// Returns the largest power of two less than or equal to `num`.
///
/// Panics if `num == 0`
fn prev_power_of_two(num: u16) -> u16 {
    if num == 0 { panic!(); }
    if num.is_power_of_two() { return num; }
    num.next_power_of_two() >> 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bsearch_non_power_of_two1() {
        let expected = BSearch { len: 160, search_range: 128, entry_selector: 3, range_shift: 32 };
        let actual = BSearch::from(10, 16);
        assert_eq!(actual, expected);
    }

    #[test]
    fn bsearch_non_power_of_two2() {
        let expected = BSearch { len: 10, search_range: 8, entry_selector: 2, range_shift: 2 };
        let actual = BSearch::from(5, 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn bsearch_power_of_two1() {
        let expected = BSearch { len: 8, search_range: 8, entry_selector: 2, range_shift: 0 };
        let actual = BSearch::from(4, 2);
        assert_eq!(actual, expected);
    }
}
