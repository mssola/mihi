use rusqlite::{Connection, Result};

/// Use the given `connection` in order to initialize the database.
pub fn init(connection: Connection) -> Result<usize> {
    connection.execute(
        r#"
CREATE TABLE IF NOT EXISTS "words" (
       "id" integer PRIMARY KEY AUTOINCREMENT NOT NULL,
       "particle" varchar,
       "enunciated" varchar,
       "declension_id" integer,
       "conjugation_id" integer,
       "kind" varchar,
       "category" integer,
       "regular" boolean DEFAULT 1,
       "locative" boolean DEFAULT 0,
       "gender" integer,
       "created_at" datetime(6) NOT NULL,
       "updated_at" datetime(6) NOT NULL,
       "suffix" varchar,
       "language_id" integer NOT NULL,
       "succeeded" integer DEFAULT 0 NOT NULL,
       "steps" integer DEFAULT 0 NOT NULL,
       "translation" jsonb DEFAULT '{}',
       "pending" boolean DEFAULT 0,
       "flags" jsonb DEFAULT '{}',
       "weight" integer DEFAULT 0 NOT NULL,

       CHECK (weight >= 0 AND weight <= 10),

       FOREIGN KEY ("conjugation_id") REFERENCES "conjugations" ("id"),
       FOREIGN KEY ("declension_id") REFERENCES "declensions" ("id")
);
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE UNIQUE INDEX IF NOT EXISTS "index_words_on_enunciated" ON "words" ("enunciated");
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE TABLE IF NOT EXISTS "exercises" (
       "id" integer PRIMARY KEY AUTOINCREMENT NOT NULL,
       "title" varchar NOT NULL,
       "enunciate" text NOT NULL,
       "solution" text NOT NULL,
       "lessons" text NOT NULL,
       "kind" integer DEFAULT 0,
       "created_at" datetime(6) NOT NULL,
       "updated_at" datetime(6) NOT NULL
);
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE UNIQUE INDEX IF NOT EXISTS "index_exercises_on_title" ON "exercises" ("title");
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE TABLE IF NOT EXISTS "tags" (
       "id" integer PRIMARY KEY AUTOINCREMENT NOT NULL,
       "name" varchar NOT NULL,
       "created_at" datetime(6) NOT NULL,
       "updated_at" datetime(6) NOT NULL
);
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE TABLE IF NOT EXISTS "tag_associations" (
       "id" integer PRIMARY KEY AUTOINCREMENT NOT NULL,
       "word_id" integer NOT NULL,
       "tag_id" integer NOT NULL,
       "created_at" datetime(6) NOT NULL,
       "updated_at" datetime(6) NOT NULL
);
"#,
        (),
    )?;

    connection.execute(
        r#"
CREATE UNIQUE INDEX IF NOT EXISTS "index_tags_on_name" ON "tags" ("name");
CREATE UNIQUE INDEX IF NOT EXISTS "word_tag_unique" ON tag_associations (word_id, tag_id);
"#,
        (),
    )?;

    Ok(0)
}
