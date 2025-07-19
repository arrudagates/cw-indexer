use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
};

// A type alias for the connection pool.
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Establishes and returns a connection pool to the `PostgreSQL` database.
///
/// # Panics
///
/// The function will panic if it fails to create the connection pool.
pub fn establish_connection_pool(database_url: &String) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let mut builder = r2d2::Pool::builder();

    if cfg!(test) {
        builder = builder.connection_timeout(std::time::Duration::new(1, 0));
    }

    builder
        .build(manager)
        .expect("Failed to create database connection pool.")
}
