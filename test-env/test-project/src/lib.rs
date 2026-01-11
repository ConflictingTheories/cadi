//! Minimal CADI project

/// Hello world function
pub fn hello() -> &'static str {
    "Hello from CADI!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from CADI!");
    }
}
