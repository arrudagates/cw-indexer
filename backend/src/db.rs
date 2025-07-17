use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
};
use std::env;

// A type alias for the connection pool.
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Establishes and returns a connection pool to the `PostgreSQL` database.
///
/// This function reads the `DATABASE_URL` from the environment variables,
/// creates a connection manager, and builds a thread-safe pool.
///
/// # Panics
///
/// The function will panic if the `DATABASE_URL` is not set in the environment
/// or if it fails to create the connection pool, as the application cannot
/// run without a database connection.
pub fn establish_connection_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool.")
}
