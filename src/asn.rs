use rasn::{
    ber::enc::Error,
    types::{Any, Oid, PrintableString},
};
use rasn_pkix::{AttributeTypeAndValue, RdnSequence, RelativeDistinguishedName};

pub fn encode_identity(
    organization: &str,
    common_name: &str,
    serial_number: &str,
) -> Result<String, Error> {
    let rdn = RdnSequence::from([
        RelativeDistinguishedName::from([AttributeTypeAndValue {
            r#type: Oid::JOINT_ISO_ITU_T_DS_ATTRIBUTE_TYPE_ORGANISATION_NAME.into(),
            value: Any::new(rasn::der::encode(&PrintableString::new(
                organization.to_string(),
            ))?),
        }]),
        RelativeDistinguishedName::from([AttributeTypeAndValue {
            r#type: Oid::JOINT_ISO_ITU_T_DS_ATTRIBUTE_TYPE_COMMON_NAME.into(),
            value: Any::new(rasn::der::encode(&PrintableString::new(
                common_name.to_string(),
            ))?),
        }]),
        RelativeDistinguishedName::from([AttributeTypeAndValue {
            r#type: Oid::JOINT_ISO_ITU_T_DS_ATTRIBUTE_TYPE_SERIAL_NUMBER.into(),
            value: Any::new(rasn::der::encode(&PrintableString::new(
                serial_number.to_string(),
            ))?),
        }]),
    ]);
    Ok(format!("asn1dn:#{}", hex::encode(rasn::der::encode(&rdn)?)))
}

#[cfg(test)]
mod test {
    #[test]
    fn encode() {
        let identity = super::encode_identity("acme organization", "some server", "0").unwrap();
        assert_eq!(
            identity,
            "asn1dn:#303e311a3018060355040a131161636d65206f7267616e697a6174696f6e311430120603550403130b736f6d6520736572766572310a30080603550405130130",
        );
    }
}
