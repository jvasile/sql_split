# sql_split

Split a string into individual sqlite sql statements.

This package is a library that contains some routines for managing
multiple sql statements in a string.  `sqlite` silently ignores
multiple statements when it only expects one.  The popular
`rusqlite` also [does
this](https://github.com/rusqlite/rusqlite/issues/1147).

```rust
use sql_split::split;
use rusqlite::{Connection, Result};

let conn = Connection::open_in_memory().expect("Can't open db in memory");
let sql = "CREATE TABLE foo (bar text); CREATE TABLE meep (moop text)";
for s in split(sql) {
    conn.execute(&s, []).expect("Can't write to the db");
}
```

In addition to basic sql statement splitting, there are functions
to count the number of statements, a short-cutting function that
tries to quickly tell you if you have more than one statement, and
a short-cutting function that only tries to split out the first n
statements.

License: MIT
