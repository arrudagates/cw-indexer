use super::environment::TestDb;
use crate::db::establish_connection_pool;

#[test]
fn test_establish_connection_pool() {
    // Create a temporary, isolated database for this test.
    let db = TestDb::new();

    // Use the URL of the temporary database to establish a new connection pool.
    let pool = establish_connection_pool(&db.db_url);

    // Assert that we can successfully get a connection from the pool.
    // This verifies that the pool was configured and built correctly.
    assert!(
        pool.get().is_ok(),
        "Failed to get a connection from the new pool."
    );
}

#[test]
#[should_panic(expected = "Failed to create database connection pool.")]
fn test_establish_connection_pool_with_invalid_url() {
    // This test ensures that the function panics with a specific message
    // when provided with an invalid database URL, as expected.
    let invalid_url = "postgres://invalid".to_string();
    establish_connection_pool(&invalid_url);
}
