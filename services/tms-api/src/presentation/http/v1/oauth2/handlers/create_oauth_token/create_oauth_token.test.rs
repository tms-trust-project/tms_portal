use super::GrantType;

#[test]
fn test_grant_type_from_str() {
    assert_eq!(GrantType::try_from("authorization_code").unwrap(), GrantType::AuthorizationCode);
    assert_eq!(GrantType::try_from("AuthoRization_code").unwrap(), GrantType::AuthorizationCode);
    assert_eq!(GrantType::try_from("REFRESH_TOKEN").unwrap(), GrantType::RefreshToken);
    assert_eq!(GrantType::try_from("refresh_token").unwrap(), GrantType::RefreshToken);
    let result = GrantType::try_from("bogus_value");
    assert!(result.is_err());
}
