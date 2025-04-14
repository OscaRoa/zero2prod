use validator::ValidateEmail;

#[derive(Debug)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(s: String) -> Result<EmailAddress, String> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid email."))
        }
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::EmailAddress;
    use claims::assert_err;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use quickcheck::Gen;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn invalid_emails_rejected() {
        let invalid_emails = vec!["", "namemail.com", "@mail.com"];
        for invalid_email in invalid_emails {
            assert_err!(EmailAddress::parse(invalid_email.to_string()));
        }
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);

            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        EmailAddress::parse(valid_email.0).is_ok()
    }
}
