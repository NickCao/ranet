use openssl::error::ErrorStack;

pub fn private_key_to_public(pem: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    let private_key = openssl::pkey::PKey::private_key_from_pem(pem)?;
    let public_key = private_key.public_key_to_pem()?;
    Ok(public_key)
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

        let public_key = super::private_key_to_public(private_key.as_bytes()).unwrap();

        assert_eq!(
            public_key,
            indoc! {"
            -----BEGIN PUBLIC KEY-----
            MCowBQYDK2VwAyEA29QaBk/rDPEAeC0nkc4agVCCCPh+D5eco9NoEX4CljU=
            -----END PUBLIC KEY-----
            "}
            .as_bytes()
        );
    }
}
