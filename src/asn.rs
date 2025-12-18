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
        insta::assert_yaml_snapshot!(identity);
    }
}
