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
use std::cmp::min;

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
/// Limitations:
///
/// The output from `split` is meant for the sqlite engine, not
/// humans.  It seeks to preserve semantics, not form.  This func will
/// strip comments because doing so simplifies parsing and also it is
/// sometimes hard to know which statement a comment should attach to.
///
/// Because we strip comments, there is one known case where
/// sql_split's behavior differs from sqlite's.  If you start a
/// .command and then start a multi-line block comment before you
/// terminate the .command with a newline, sqlite will throw an error.
/// We just remove that error and give you the .command as its own
/// statement.
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
    split_n(sql, None)
}

/// Split first N statements from a multi-statement sql string into a Vec<String>
///
/// SQL is an &str containing some sql statements
/// N is an Option<usize>.  If present, return up to N statements.
/// ```rust
/// use sql_split::split_n;
/// use rusqlite::{Connection, Result};
///
/// let sql = "CREATE TABLE foo (bar text); CREATE TABLE meep (moop text)";
/// assert_eq!(split_n(sql, Some(1)), vec!["CREATE TABLE foo (bar text);"]);
/// ```
pub fn split_n(sql: &str, n: Option<usize>) -> Vec<String> {
    let mut ret: Vec<String> = vec![];
    let mut statement: String = "".to_owned();
    let mut encloser: Option<char> = None;
    let mut last_ch = ' ';
    let mut in_line_comment: bool = false;
    let mut in_block_comment: bool = false;
    let mut in_dot_command: bool = false;
    let mut record_statement: bool = false;
    for ch in sql.chars() {
        if !in_line_comment && !in_block_comment {
            statement.push(ch);
        }
        match encloser {
            Some(e) => {
                if ch == ']' || e == ch {
                    encloser = None;
                }
            }
            None => match ch {
                '.' => {
                    if statement.len() == 1 && !in_block_comment {
                        in_dot_command = true;
                    }
                }
                '*' => {
                    if !in_line_comment && !in_block_comment {
                        if last_ch == '/' {
                            in_block_comment = true;
                            // unpush the /*
                            statement.pop().unwrap();
                            statement.pop().unwrap();

                            // This one might be controversial.  If
                            // you start a /*comment*/ while in a
                            // .command, sqlite will throw an error.
                            // We are stripping comments, though, so
                            // we can't really reproduce that.  Here,
                            // I just end the .command and delete the
                            // comment.
                            if in_dot_command {
                                record_statement = true;
                            }
                        }
                    }
                }
                '/' => {
                    if in_block_comment && last_ch == '*' {
                        in_block_comment = false;
                    }
                }
                '\n' => {
                    if in_dot_command {
                        record_statement = true;
                        in_dot_command = false;
                    }
                    if in_line_comment {
                        in_line_comment = false;
                    }
                }
                '-' => {
                    if !in_line_comment && !in_block_comment {
                        if last_ch == '-' {
                            in_line_comment = true;

                            // unpush the --
                            statement.pop().unwrap();
                            statement.pop().unwrap();
                        }
                    }
                }
                ';' => {
                    if !in_line_comment && !in_block_comment {
                        record_statement = true;
                    }
                }
                '[' | '"' | '\'' | '`' => encloser = Some(ch),
                _ => {}
            },
        }
        last_ch = ch;

        if record_statement {
            statement = statement.trim().to_owned();

            // Push statement if not empty
            if statement != ";" && !statement.is_empty() {
                ret.push(statement.to_owned());
                if let Some(n) = n {
                    if ret.len() >= n {
                        break;
                    }
                }
            }
            statement = "".to_owned();
            record_statement = false;
        }
    }

    // Capture anything left over, in case sql doesn't end in
    // semicolon.  Note that if we `break` in the above loop,
    // statement might not be empty, and all the sql left will get
    // tacked on to ret.
    if statement.trim().len() != 0 {
        ret.push(statement.trim().to_string())
    }

    match n {
        Some(n) => ret[0..min(n, ret.len())].to_vec(),
        None => ret,
    }
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

/// Return true iff SQL contains multiple sql statements
///
/// ```rust
/// use sql_split::is_multiple;
///
/// assert!(!is_multiple("CREATE Table foo (bar text)"));
/// assert!(is_multiple("CREATE Table foo (bar text);INSERT blah blah blah"));
/// assert!(!is_multiple("-- line comment\nCREATE Table foo (bar text)"));
///
/// ```
pub fn is_multiple(sql: &str) -> bool{
    split_n(sql, Some(2)).len() > 1
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
    }

    #[test]
    fn test_split_comments() {
        assert_eq!(
            split("SELECT * FROM foo; -- trailing comments are fine"),
            vec!["SELECT * FROM foo;"]
        );
        assert_eq!(
            split("SELECT * FROM foo -- trailing comments are fine"),
            vec!["SELECT * FROM foo"],
            "Fail trailing -- comment w/ no semicolon"
        );
        assert_eq!(
            split("SELECT * FROM foo; -- trailing comments are fine\n Another statement"),
            vec!["SELECT * FROM foo;", "Another statement",]
        );
        assert_eq!(
            split("SELECT * FROM foo; -- trailing ; comments ; are ; fine"),
            vec!["SELECT * FROM foo;"],
            "trailing -- comment w/ multiple semicolons"
        );
        assert_eq!(
            split("SELECT * FROM foo /* trailing comments are fine */"),
            vec!["SELECT * FROM foo"],
            "trailing block comment, no semicolon"
        );
        assert_eq!(
            split("SELECT * FROM foo; /* trailing comments are fine */"),
            vec!["SELECT * FROM foo;"],
            "trailing block comment"
        );
        assert_eq!(
            split("CREATE TABLE foo (\nbar text -- describe bar\nbaz int -- how many baz\n);"),
            vec!["CREATE TABLE foo (\nbar text baz int );"],
            "multiline statement with --comments interspersed"
        );
        assert_eq!(
            split("SELECT * FROM foo /* block comment mid-statement */ WHERE blah blah blah"),
            vec!["SELECT * FROM foo  WHERE blah blah blah"],
            "block comment mid-statement"
        );
        assert_eq!(
            split("SELECT * FROM foo /* multiline\n\ncomments are fine mid-statement */ WHERE blah blah blah"),
            vec!["SELECT * FROM foo  WHERE blah blah blah"]
        );
        assert!(split("-- Start with a comment;SELECT * FROM foo;").is_empty());
        assert_eq!(
            split("-- Start with a comment\nSELECT * FROM foo;"),
            vec!["SELECT * FROM foo;"],
            "-- comment didn't know where to stop"
        );
    }

    #[test]
    fn split_dot_commant() {
        assert_eq!(split(".dump"), vec![".dump"], "Failed at basic dot command");
        assert_eq!(
            split(".dump --with a comment"),
            vec![".dump"],
            "Failed at dot command w/ --comment"
        );
        assert_eq!(
            split(".dump\nDROP TABLE foo"),
            vec![".dump", "DROP TABLE foo"],
            ".dump + statement"
        );
        assert_eq!(
            split("SELECT * FROM foo;.dump\nDROP TABLE foo"),
            vec!["SELECT * FROM foo;", ".dump", "DROP TABLE foo"],
            "statement + .dump + statement"
        );
        assert_eq!(
            split(".dump /* block comment */ DROP TABLE foo"),
            vec![".dump", "DROP TABLE foo"],
            ".dump + /*block comment*/ + statement"
        );
        assert_eq!(
            split(".dump /* multiline \n comment */ DROP TABLE foo"),
            vec![".dump", "DROP TABLE foo"],
            ".dump + /*multiline comment*/ + statement"
        );
    }

    #[test]
    fn split_empty_statements() {
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
    fn test_split_n() {
        assert_eq!(
            split_n("CREATE TABLE foo (bar text)", Some(1)),
            vec!["CREATE TABLE foo (bar text)"]
        );
        assert_eq!(
            split_n("CREATE TABLE foo (bar text) -- table creation", Some(1)),
            vec!["CREATE TABLE foo (bar text)"]
        );
        assert_eq!(
            split_n(
                "Try more than 1;CREATE TABLE foo (bar text) -- table creation",
                Some(2)
            ),
            vec!["Try more than 1;", "CREATE TABLE foo (bar text)"]
        );
    }

    #[test]
    fn test_count() {
        assert_eq!(count(""), 0, "Failed at empty sql");
        assert_eq!(
            count("CREATE TABLE foo (bar text)"),
            1,
            "Failed at one basic statement"
        );
        assert_eq!(
            count("CREATE TABLE foo (bar text); INSERT into foo (bar) VALUES ('hi')"),
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
