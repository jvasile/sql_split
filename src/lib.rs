//! Split a string into individual sql statements.
//!
//! This package is a library that contains some routines for managing
//! multiple sql statements in a string.  `sqlite` silently ignores
//! multiple statements when it only expects one.  The popular
//! `rusqlite` also [has this
//! flaw](https://github.com/rusqlite/rusqlite/issues/1147).
//!
//! Although this package aims at sqlite, it would not be difficult to
//! extend it to other sql dialects.
//!
//!
//! ```rust
//! use sql_split::split;
//!
//! split("CREATE TABLE foo ({bar: text});");
//! ```
#![warn(missing_docs)]



/// Split a string into individual sql statements.
///
/// ```rust
/// use sql_split::split;
///
/// ```
pub fn split(sql: &str) -> Vec<String> {
    vec![]
}

/// Count statements in a string of sqlite sql
///
/// ```rust
/// use sql_split::count;
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
	assert_eq!(split(""),vec![""]);
	assert_eq!(split("CREATE TABLE foo (bar: text)"), vec!["CREATE TABLE foo (bar: text)"]); // trailing semi-colon is optional
	assert_eq!(split("CREATE TABLE foo (bar: text);"), vec!["CREATE TABLE foo (bar: text);"]);
	assert_eq!(split("CREATE TABLE foo (bar: text); INSERT into foo (bar) VALUES ('hi')"), vec!["CREATE TABLE foo (bar: text)", " INSERT into foo (bar) VALUES ('hi')"]);
	assert_eq!(split("invalid sql; but we don't care because we don't really parse it;"), vec!["invalid sql", " but we don't care because we don't really parse it;"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES ('semicolon in string: ;')"), vec!["INSERT INTO foo (bar) VALUES ('semicolon in string: ;')"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES (\"semicolon in double-quoted string: ;\")"), vec!["INSERT INTO foo (bar) VALUES (\"semicolon in double-quoted string: ;\")"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES (`semicolon in backtick string: ;`)"), vec!["INSERT INTO foo (bar) VALUES (`semicolon in backtick string: ;`)"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES ('interior quote and semicolon in string: ;''')"), vec!["INSERT INTO foo (bar) VALUES ('interior quote and semicolon in string: ;''')"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES (\"interior quote and semicolon in double-quoted string: ;\"\"\")"), vec!["INSERT INTO foo (bar) VALUES (\"interior quote and semicolon in double-quoted string: ;\"\"\")"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES (`interior quote and semicolon in backtick string: ;```)"), vec!["INSERT INTO foo (bar) VALUES (`interior quote and semicolon in backtick string: ;```)"]);
	assert_eq!(split("INSERT INTO foo (bar) VALUES (`semicolon after interior quote ``;`)"), vec!["INSERT INTO foo (bar) VALUES (`semicolon after interior quote ``;`)"]);
	assert_eq!(split("CREATE TABLE [foo;bar] (bar: text); INSERT into foo (bar) VALUES ('hi')"), vec!["CREATE TABLE [foo;bar] (bar: text)", " INSERT into foo (bar) VALUES ('hi')"]); // brackets are ok for identifiers in sqlite
	assert_eq!(split("SELECT * FROM foo; -- trailing comments are fine"), vec!["SELECT * FROM foo; -- trailing comments are fine"]);
	assert_eq!(split("SELECT * FROM foo; /* trailing comments are fine */"), vec!["SELECT * FROM foo; /* trailing comments are fine */"]);
    }


    #[test]
    fn test_count() {
	assert_eq!(count(""),0);
	assert_eq!(count("CREATE TABLE foo (bar: text)"),1);
	assert_eq!(count("CREATE TABLE foo (bar: text);"),1); // trailing semi-colon is optional
	assert_eq!(count("CREATE TABLE foo (bar: text); INSERT into foo (bar) VALUES ('hi')"),2);
	assert_eq!(count("invalid sql; but we don't care because we don't really parse it;"),2);
	assert_eq!(count("INSERT INTO foo (bar) VALUES ('semicolon in string: ;')"), 1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES (\"semicolon in double-quoted string: ;\")"),1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES (`semicolon in backtick string: ;`)"),1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES ('interior quote and semicolon in string: ;''')"), 1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES (\"interior quote and semicolon in double-quoted string: ;\"\"\")"),1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES (`interior quote and semicolon in backtick string: ;```)"),1);
	assert_eq!(count("INSERT INTO foo (bar) VALUES (`semicolon after interior quote ``;`)"),1);
	assert_eq!(count("CREATE TABLE [foo;bar] (bar: text); INSERT into foo (bar) VALUES ('hi')"),2); // brackets are ok for identifiers in sqlite
	assert_eq!(count("SELECT * FROM foo; -- trailing comments are fine"), 1);
	assert_eq!(count("SELECT * FROM foo; /* trailing comments are fine */"), 1);
    }



}
