const ALPHABET: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];

pub fn generate_uid() -> String {
    nanoid::nanoid!(10, ALPHABET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_is_ten_chars() {
        let uid = generate_uid();
        assert_eq!(uid.len(), 10);
    }

    #[test]
    fn uid_is_alphanumeric() {
        let uid = generate_uid();
        assert!(uid.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn two_uids_differ() {
        let a = generate_uid();
        let b = generate_uid();
        assert_ne!(a, b);
    }
}
