use crate::get_connection;
use rusqlite::params;

/// The exercise kinds supported by this application.
#[derive(Clone, Copy, Debug, Default)]
pub enum ExerciseKind {
    #[default]
    Pensum = 0,
    Translation,
    Transformation,
    Numerical,
}

impl std::fmt::Display for ExerciseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pensum => write!(f, "Pensum"),
            Self::Translation => write!(f, "Translation"),
            Self::Transformation => write!(f, "Transformation"),
            Self::Numerical => write!(f, "Numerical"),
        }
    }
}

impl TryFrom<isize> for ExerciseKind {
    type Error = &'static str;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Pensum),
            1 => Ok(Self::Translation),
            2 => Ok(Self::Transformation),
            3 => Ok(Self::Numerical),
            _ => Err("unknonwn exercise kind"),
        }
    }
}

impl TryFrom<&str> for ExerciseKind {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pensum" => Ok(Self::Pensum),
            "translation" => Ok(Self::Translation),
            "transformation" => Ok(Self::Transformation),
            "numerical" => Ok(Self::Numerical),
            _ => Err("unknonwn exercise kind. Available: pensum, translation, transformation and numerical"),
        }
    }
}

/// Exercise as laid out in the 'exercises' table.
#[derive(Clone, Debug, Default)]
pub struct Exercise {
    pub id: i32,
    pub title: String,
    pub enunciate: String,
    pub solution: String,
    pub lessons: String,
    pub kind: ExerciseKind,
}

/// Creates the given exercise into the database.
pub fn create_exercise(exercise: Exercise) -> Result<(), String> {
    let conn = get_connection()?;
    match conn.execute(
        "INSERT INTO exercises (title, enunciate, solution, lessons, kind, \
                                updated_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
        params![
            exercise.title,
            exercise.enunciate,
            exercise.solution,
            exercise.lessons,
            exercise.kind as isize,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not create '{}': {}", exercise.title, e)),
    }
}

pub fn select_by_title(filter: Option<String>) -> Result<Vec<String>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match filter {
        Some(filter) => {
            stmt = conn
                .prepare(
                    "SELECT title FROM exercises WHERE title LIKE ('%' || ?1 || '%') ORDER BY title",
                )
                .unwrap();
            stmt.query([filter.as_str()]).unwrap()
        }
        None => {
            stmt = conn
                .prepare("SELECT title FROM exercises ORDER BY title")
                .unwrap();
            stmt.query([]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(row.get::<usize, String>(0).unwrap());
    }
    Ok(res)
}

pub fn find_exercise_by_title(title: &str) -> Result<Exercise, String> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, enunciate, solution, lessons, kind  \
             FROM exercises \
             WHERE title = ?1",
        )
        .unwrap();
    let mut it = stmt.query([title]).unwrap();

    match it.next() {
        Err(_) => Err("no exercises were found with this title".to_string()),
        Ok(rows) => match rows {
            Some(row) => Ok(Exercise {
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                enunciate: row.get(2).unwrap(),
                solution: row.get(3).unwrap(),
                lessons: row.get(4).unwrap(),
                kind: row.get::<usize, isize>(5).unwrap().try_into()?,
            }),
            None => Err("no exercises were found with this title".to_string()),
        },
    }
}

/// Updates the given exercise.
pub fn update_exercise(exercise: Exercise) -> Result<(), String> {
    if exercise.id == 0 {
        return Err("invalid exercise to update; seems it has not been created before".to_string());
    }

    let conn = get_connection()?;

    match conn.execute(
        "UPDATE exercises \
         SET title = ?2, enunciate = ?3, solution = ?4, lessons = ?5, kind = ?6, \
             updated_at = datetime('now') \
         WHERE id = ?1",
        params![
            exercise.id,
            exercise.title,
            exercise.enunciate,
            exercise.solution,
            exercise.lessons,
            exercise.kind as isize,
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", exercise.title, e)),
    }
}

/// Updates the 'updated_at' column for an exercise.
pub fn touch_exercise(exercise: Exercise) -> Result<(), String> {
    if exercise.id == 0 {
        return Err("invalid exercise to update; seems it has not been created before".to_string());
    }

    let conn = get_connection()?;

    match conn.execute(
        "UPDATE exercises \
         SET updated_at = datetime('now') \
         WHERE id = ?1",
        params![exercise.id],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not update '{}': {}", exercise.title, e)),
    }
}

/// Delete an exercise from the database.
pub fn delete_exercise(title: &str) -> Result<(), String> {
    let conn = get_connection()?;

    match conn.execute("DELETE FROM exercises WHERE title = ?1", params![title]) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("could not remove '{title}': {e}")),
    }
}

// Get a list of exercises sorted by relevance. A maximum of `limit` exercises
// will be returned, and you can also specify to filter the returned exercises
// by `kind`.
pub fn select_relevant_exercises(
    kind: Option<ExerciseKind>,
    limit: isize,
) -> Result<Vec<Exercise>, String> {
    let conn = get_connection()?;

    let mut stmt;
    let mut it = match kind {
        Some(kind) => {
            stmt = conn
                .prepare(
                    "SELECT id, title, enunciate, solution, lessons, kind  \
                     FROM exercises \
                     WHERE kind = ?1 \
                     ORDER BY updated_at DESC \
                     LIMIT ?2",
                )
                .unwrap();
            stmt.query([kind as isize, limit]).unwrap()
        }
        None => {
            stmt = conn
                .prepare(
                    "SELECT id, title, enunciate, solution, lessons, kind  \
                     FROM exercises \
                     ORDER BY updated_at DESC \
                     LIMIT ?1",
                )
                .unwrap();
            stmt.query([limit]).unwrap()
        }
    };

    let mut res = vec![];
    while let Some(row) = it.next().unwrap() {
        res.push(Exercise {
            id: row.get(0).unwrap(),
            title: row.get(1).unwrap(),
            enunciate: row.get(2).unwrap(),
            solution: row.get(3).unwrap(),
            lessons: row.get(4).unwrap(),
            kind: row.get::<usize, isize>(5).unwrap().try_into()?,
        });
    }
    Ok(res)
}
