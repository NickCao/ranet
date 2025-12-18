use ed25519_dalek::{
    pkcs8::{DecodePrivateKey, EncodePublicKey, Error},
    SigningKey,
};
use x509_cert::der::pem::LineEnding;

pub fn private_key_to_public(pem: &str) -> Result<String, Error> {
    Ok(SigningKey::from_pkcs8_pem(pem)?
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)?)
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    #[test]
    fn private_key_to_public() {
        let private_key = indoc! {"
            -----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIHJiQXiRUBti6HjAxgz3p2ZwIJNjPT/P5iuYPYLhOylO
            -----END PRIVATE KEY-----
        "};

        let public_key = super::private_key_to_public(private_key).unwrap();

        insta::assert_yaml_snapshot!(public_key);
    }
}
