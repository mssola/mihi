use crate::word::Word;
use rusqlite::params;

/// A tag which can be associated with multiple words. It is mapped in the
/// database via the 'tags' and 'tag_associations' tables.
#[derive(Clone, Debug)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

// Needed for inquire's (Multi)Select.
impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Returns a vector with the names for tags that match the given `filter`, or
/// all of them if None is passed as the filter.
pub fn select_tag_names(filter: &Option<String>) -> Result<Vec<String>, String> {
    let conn = crate::get_connection()?;

    let mut stmt;
    let mut it = match filter {
        Some(filter) => {
            stmt = conn
                .prepare("SELECT name FROM tags WHERE name LIKE ('%' || ?1 || '%') ORDER BY name")
                .unwrap();
            stmt.query([filter.as_str()]).unwrap()
        }
        None => {
            stmt = conn.prepare("SELECT name FROM tags ORDER BY name").unwrap();
            stmt.query([]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(row.get::<usize, String>(0).unwrap());
    }
    Ok(res)
}

/// Select all tags for the given `word`. If None is provided, then all tags
/// from the database are returned.
pub fn select_tags_for(word: Option<i32>) -> Result<Vec<Tag>, String> {
    let conn = crate::get_connection()?;

    let mut stmt;
    let mut it = match word {
        Some(id) => {
            stmt = conn
                .prepare(
                    "SELECT t.id, t.name \
                     FROM tags t \
                     JOIN tag_associations ta ON t.id = ta.tag_id \
                     JOIN words w ON w.id = ta.word_id \
                     WHERE w.id = ?1 \
                     ORDER BY t.name",
                )
                .unwrap();
            stmt.query([id]).unwrap()
        }
        None => {
            stmt = conn
                .prepare("SELECT id, name FROM tags ORDER BY name")
                .unwrap();
            stmt.query([]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Tag {
            id: row.get::<usize, i32>(0).unwrap(),
            name: row.get::<usize, String>(1).unwrap(),
        });
    }
    Ok(res)
}

/// Insert into the database the tag identified by the given name.
pub fn create_tag(name: &str) -> Result<(), String> {
    let conn = crate::get_connection()?;

    match conn.execute(
        "INSERT INTO tags (name, updated_at, created_at) \
         VALUES (?1, datetime('now'), datetime('now'))",
        params![name.trim()],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not create '{}': {}", name, e)),
    }
}

/// Inserts the pair of IDs into the tag_associations table.
pub fn attach_tag_to_word(tag_id: i64, word_id: i64) -> Result<(), String> {
    let conn = crate::get_connection()?;

    match conn.execute(
        "INSERT INTO tag_associations (tag_id, word_id, updated_at, created_at) \
         VALUES (?1, ?2, datetime('now'), datetime('now'))",
        params![tag_id, word_id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not attach tag: {e}")),
    }
}

/// Inserts the pair of IDs into the tag_associations table.
pub fn dettach_tags_from_word(tags: &[i32], word_id: i64) -> Result<(), String> {
    if tags.is_empty() {
        return Ok(());
    }

    let conn = crate::get_connection()?;

    match conn.execute(
        format!(
            "DELETE FROM tag_associations \
             WHERE tag_id in ({}) AND word_id = ?1",
            tags.iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .as_str(),
        params![word_id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not attach tag: {e}")),
    }
}

/// Delete the tag from the database.
pub fn delete_tag(name: &String) -> Result<(), String> {
    let conn = crate::get_connection()?;

    match conn.execute("DELETE FROM tags WHERE name = ?1", params![name.trim()]) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not remove '{name}': {e}")),
    }
}

/// Update the success and steps rates for a given word.
pub fn update_success(word: &Word, success: isize, steps: isize) -> Result<(), String> {
    let conn = crate::get_connection()?;

    match conn.execute(
        "UPDATE words \
         SET succeeded = ?1, steps = ?2, updated_at = datetime('now') \
         WHERE id = ?3",
        params![success, steps, word.id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", word.enunciated, e)),
    }
}
