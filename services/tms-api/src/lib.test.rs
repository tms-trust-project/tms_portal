use super::build_database_url;

#[test]
fn builds_the_postgres_url() {
    let url = build_database_url("tms", "secret", "localhost", "5432", "tms_db").unwrap();

    assert_eq!(url, "postgres://tms:secret@localhost:5432/tms_db");
}

#[test]
fn an_invalid_url_error_masks_the_password() {
    let error = build_database_url("tms", "s3cret-pw", "localhost", "not-a-port", "tms_db")
        .unwrap_err()
        .to_string();

    assert!(!error.contains("s3cret-pw"), "{error}");
    assert!(
        error.starts_with(
            "The database url postgres://tms:***@localhost:not-a-port/tms_db is not valid"
        ),
        "{error}"
    );
}
