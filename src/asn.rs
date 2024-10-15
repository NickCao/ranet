use const_oid::db::rfc4519::{CN, ORGANIZATION_NAME, SERIAL_NUMBER};
use x509_cert::{
    attr::AttributeTypeAndValue,
    der::asn1::{PrintableString, SetOfVec, Utf8StringRef},
    der::{Encode, Result},
    name::{RdnSequence, RelativeDistinguishedName},
};

pub fn encode_identity(
    organization: &str,
    common_name: &str,
    serial_number: &str,
) -> Result<String> {
    let name = RdnSequence(vec![
        RelativeDistinguishedName(SetOfVec::from_iter([AttributeTypeAndValue {
            oid: ORGANIZATION_NAME,
            value: Utf8StringRef::new(organization)?.into(),
        }])?),
        RelativeDistinguishedName(SetOfVec::from_iter([AttributeTypeAndValue {
            oid: CN,
            value: Utf8StringRef::new(common_name)?.into(),
        }])?),
        RelativeDistinguishedName(SetOfVec::from_iter([AttributeTypeAndValue {
            oid: SERIAL_NUMBER,
            value: (&PrintableString::new(serial_number)?).into(),
        }])?),
    ])
    .to_der()?;
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
