pub mod cfg;
pub mod exercise;
pub mod inflection;
pub mod tag;
pub mod word;

/// Get a connection to the database. Note that you can set the 'MIHI_DATABASE'
/// environment variable to define an alternative path.
pub fn get_connection() -> Result<rusqlite::Connection, String> {
    let name = &std::env::var("MIHI_DATABASE").unwrap_or("database.sqlite3".to_string());
    let path = crate::cfg::get_config_path()?.join(name);

    match rusqlite::Connection::open(&path) {
        Ok(handle) => Ok(handle),
        Err(_) => Err(format!(
            "could not fetch the database in '{}'",
            path.display()
        )),
    }
}
