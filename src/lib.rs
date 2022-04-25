//! Split a string into individual sqlite sql statements.
//!
//! This package is a library that contains some routines for managing
//! multiple sql statements in a string.  `sqlite` silently ignores
//! multiple statements when it only expects one.  The popular
//! `rusqlite` also [has this
//! flaw](https://github.com/rusqlite/rusqlite/issues/1147).
//!
//! ```rust
//! use sql_split::split;
//! use rusqlite::{Connection, Result};
//!
//! let conn = Connection::open_in_memory().expect("Can't open db in memory");
//! let sql = "CREATE TABLE foo (bar text); CREATE TABLE meep (moop text)";
//! for s in split(sql) {
//!     conn.execute(&s, []).expect("Can't write to the db");
//! }
//! ```
#![warn(missing_docs)]

/// Split a string into individual sql statements.
///
/// This func returns a Vec<String> containing individual sql
/// statements in SQL.
///
/// This func is not smart, but it's not completely dumb either.  It
/// will ignore semicolons inside enclosures recognized by `sqlite`
/// (quote, double-quote, backticks, square braces).  It will not be
/// thrown off by nested quotes.  It will remove empty statements if
/// it finds any.
///
/// This func does not know how to parse sql, and will not verify that
/// your sql is well-formed.  If you feed it invalid sql, results are
/// undefined.  For example, it will not treat handle quotes escaped
/// with a backslash as a special case, since sqlite does not
/// recognize backslash escaped quotes.
///
/// ```rust
/// use sql_split::split;
/// use rusqlite::{Connection, Result};
///
/// let conn = Connection::open_in_memory().expect("Can't open db in memory");
/// let sql = "CREATE TABLE foo (bar text); CREATE TABLE meep (moop text);";
/// for s in split(sql) {
///     conn.execute(&s, []).expect("Can't write to the db");
/// }
///
/// assert_eq!(split(sql),
///     vec![
///         "CREATE TABLE foo (bar text);",
///         "CREATE TABLE meep (moop text);",
///     ]
/// );
/// ```
pub fn split(sql: &str) -> Vec<String> {
    let mut ret: Vec<String> = vec![];
    let mut statement: String = "".to_string();
    let mut encloser: Option<char> = None;
    for ch in sql.chars() {
        statement.push(ch);
        match encloser {
            Some(e) => {
                if ch == ']' || e == ch {
                    encloser = None;
                }
            }
            None => match ch {
                ';' => {
                    statement = statement.trim().to_owned();
                    if statement != ";" {
                        // ignore empty statements
                        ret.push(statement);
                    }
                    statement = "".to_string();
                }
                '[' | '"' | '\'' | '`' => {
                    encloser = Some(ch);
                }
                _ => {}
            },
        }
    }

    // Capture anything left over, in case sql doesn't end in semicolon
    if statement.trim().len() != 0 {
        ret.push(statement.trim().to_string())
    }

    // Handle trailing comments
    if ret.len() >= 2 {
        let l = ret.last().unwrap();
        if l.starts_with("--") || l.starts_with("/*") {
            let comment: String = ret.pop().unwrap();
            let last = ret.pop().unwrap();
            ret.push(format!("{} {}", last, comment));
        }
    }

    ret
}

/// Count statements in a string of sqlite sql
///
/// This func returns the number of sql statements in an &str.
///
/// ```rust
/// use sql_split::count;
///
/// let sql = "CREATE TABLE foo (bar text); CREATE TABLE meep (moop text)";
/// assert_eq!(count(sql), 2);
///
/// ```
pub fn count(sql: &str) -> usize {
    split(sql).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        assert_eq!(
            split("CREATE TABLE foo (bar text)"),
            vec!["CREATE TABLE foo (bar text)"],
            "Trailing semi-colon is optional"
        );
        assert_eq!(
            split("CREATE TABLE foo (bar text);"),
            vec!["CREATE TABLE foo (bar text);"],
            "We preserve the semi-colons"
        );
        assert_eq!(
            split("CREATE TABLE foo (bar text); INSERT into foo (bar) VALUES ('hi')"),
            vec![
                "CREATE TABLE foo (bar text);",
                "INSERT into foo (bar) VALUES ('hi')"
            ]
        );
        assert_eq!(
            split("invalid sql; but we don't care because we don't really parse it;"),
            vec![
                "invalid sql;",
                "but we don't care because we don't really parse it;"
            ]
        );
        assert_eq!(
            split("INSERT INTO foo (bar) VALUES ('semicolon in string: ;')"),
            vec!["INSERT INTO foo (bar) VALUES ('semicolon in string: ;')"]
        );
        assert_eq!(
            split("INSERT INTO foo (bar) VALUES (\"semicolon in double-quoted string: ;\")"),
            vec!["INSERT INTO foo (bar) VALUES (\"semicolon in double-quoted string: ;\")"]
        );
        assert_eq!(
            split("INSERT INTO foo (bar) VALUES (`semicolon in backtick string: ;`)"),
            vec!["INSERT INTO foo (bar) VALUES (`semicolon in backtick string: ;`)"]
        );
        assert_eq!(
            split("INSERT INTO foo (bar) VALUES ('interior quote and semicolon in string: ;''')"),
            vec!["INSERT INTO foo (bar) VALUES ('interior quote and semicolon in string: ;''')"]
        );
        assert_eq!(split("INSERT INTO foo (bar) VALUES (\"interior quote and semicolon in double-quoted string: ;\"\"\")"), vec!["INSERT INTO foo (bar) VALUES (\"interior quote and semicolon in double-quoted string: ;\"\"\")"]);
        assert_eq!(split("INSERT INTO foo (bar) VALUES (`interior quote and semicolon in backtick string: ;```)"), vec!["INSERT INTO foo (bar) VALUES (`interior quote and semicolon in backtick string: ;```)"]);
        assert_eq!(
            split("INSERT INTO foo (bar) VALUES (`semicolon after interior quote ``;`)"),
            vec!["INSERT INTO foo (bar) VALUES (`semicolon after interior quote ``;`)"]
        );
        assert_eq!(
            split("CREATE TABLE [foo;bar] (bar: text); INSERT into foo (bar) VALUES ('hi')"),
            vec![
                "CREATE TABLE [foo;bar] (bar: text);",
                "INSERT into foo (bar) VALUES ('hi')"
            ]
        ); // brackets are ok for identifiers in sqlite
        assert_eq!(
            split("SELECT * FROM foo; -- trailing comments are fine"),
            vec!["SELECT * FROM foo; -- trailing comments are fine"]
        );
        assert_eq!(
            split("SELECT * FROM foo; /* trailing comments are fine */"),
            vec!["SELECT * FROM foo; /* trailing comments are fine */"]
        );

        assert_eq!(
            split("hi;internal;;;;;bye;"),
            vec!["hi;", "internal;", "bye;"],
            "Failing at removing empty statements"
        );

        assert_eq!(
            split("trailing newlines;\n\n\n\n\n\n\n\n;\n\n\n\n\n"),
            vec!["trailing newlines;"]
        );
    }

    #[test]
    fn test_count() {
        assert_eq!(count(""), 0, "Failed at empty sql");
        assert_eq!(
            count("CREATE TABLE foo (bar: text)"),
            1,
            "Failed at one basic statement"
        );
        assert_eq!(
            count("CREATE TABLE foo (bar: text); INSERT into foo (bar) VALUES ('hi')"),
            2,
            "Failed at two basic statements"
        );
        assert_eq!(
            count("SELECT * FROM foo; -- trailing comments are fine"),
            1,
            "Failed at -- comment"
        );
        assert_eq!(
            count("SELECT * FROM foo; /* trailing comments are fine */"),
            1,
            "Failed at /*comment*/"
        );
    }
}
