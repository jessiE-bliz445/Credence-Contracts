#![no_std]

pub fn is_valid_bond(amount: i128) -> bool {
    amount > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_positive() {
        assert!(is_valid_bond(1));
    }

    #[test]
    fn invalid_zero() {
        assert!(!is_valid_bond(0));
    }

    #[test]
    fn invalid_negative() {
        assert!(!is_valid_bond(-5));
    }
}
