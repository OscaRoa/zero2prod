use rand::distr::{Alphanumeric, SampleString};
use rand::prelude::ThreadRng;
use rand::rng;

const TOKEN_LENGTH: usize = 25;

#[derive(Debug)]
pub enum TokenError {
    InvalidLength { expected: usize, actual: usize },
    InvalidFormat,
}

#[derive(Debug)]
pub struct SubscriptionToken(pub String);

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriptionToken {
    pub fn new() -> Self {
        Self::generate_token_with_rng(&mut rng())
    }

    pub fn parse(s: &str) -> Result<Self, TokenError> {
        let chars_count = s.len();
        if chars_count != TOKEN_LENGTH {
            return Err(TokenError::InvalidLength {
                expected: TOKEN_LENGTH,
                actual: chars_count,
            });
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(TokenError::InvalidFormat);
        }

        Ok(Self(s.to_string()))
    }

    fn generate_token_with_rng(rng: &mut ThreadRng) -> Self {
        let token = Alphanumeric.sample_string(rng, TOKEN_LENGTH);
        Self::parse(&token).unwrap()
    }
}

impl Default for SubscriptionToken {
    fn default() -> Self {
        SubscriptionToken::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err;

    #[test]
    fn test_token_generation() {
        let token = SubscriptionToken::new();
        assert_eq!(token.as_ref().len(), TOKEN_LENGTH);
        assert!(token.as_ref().chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_valid_token_parsing() {
        let valid_token = "5J91vXYKj2xP8LmN3qRt4wZhA";
        assert!(SubscriptionToken::parse(valid_token).is_ok());
    }

    #[test]
    fn test_invalid_length_token() {
        let short_token = "abc";
        let result = SubscriptionToken::parse(short_token);
        assert!(matches!(
            result,
            Err(TokenError::InvalidLength {
                expected: TOKEN_LENGTH,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_invalid_format_token() {
        let invalid_token = "!@#$%^&*()?><:{}[]";
        assert_err!(SubscriptionToken::parse(invalid_token));
    }
}
