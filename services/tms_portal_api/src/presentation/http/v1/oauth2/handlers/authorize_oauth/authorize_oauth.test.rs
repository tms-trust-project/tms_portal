use super::ResponseType;

#[test]
fn test_response_type_from_str() {
    assert_eq!(ResponseType::try_from("code").unwrap(), ResponseType::Code);
    assert_eq!(ResponseType::try_from("CODE").unwrap(), ResponseType::Code);
    assert_eq!(ResponseType::try_from("CodE").unwrap(), ResponseType::Code);
    let result = ResponseType::try_from("bogus_value");
    assert!(result.is_err());
}
