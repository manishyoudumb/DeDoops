use textdistance::str::levenshtein;

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    levenshtein(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("flaw", "lawn"), 2);
        assert_eq!(levenshtein_distance("gumbo", "gambol"), 2);
        assert_eq!(levenshtein_distance("book", "back"), 2);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("rust", "rust"), 0);
    }
} 