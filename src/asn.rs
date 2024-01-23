use openssl::error::ErrorStack;

pub fn encode_identity(
    organization: &str,
    common_name: &str,
    serial_number: &str,
) -> Result<String, ErrorStack> {
    let mut b = openssl::x509::X509NameBuilder::new()?;
    b.append_entry_by_text("O", organization)?;
    b.append_entry_by_text("CN", common_name)?;
    b.append_entry_by_text("serialNumber", serial_number)?;
    let name = b.build().to_der()?;
    Ok(format!("asn1dn:#{}", hex::encode(name)))
}

#[cfg(test)]
mod test {
    #[test]
    fn encode_identity() {
        let identity = super::encode_identity("acme organization", "some server", "0").unwrap();
        assert_eq!(
            identity,
            "asn1dn:#303e311a3018060355040a0c1161636d65206f7267616e697a6174696f6e3114301206035504030c0b736f6d6520736572766572310a30080603550405130130",
        );
    }
}
