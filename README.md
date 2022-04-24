# sql_split

Split a string into individual sqlite sql statements.

This package is a library that contains some routines for managing
multiple sql statements in a string.  `sqlite` silently ignores
multiple statements when it only expects one.  The popular
`rusqlite` also [has this
flaw](https://github.com/rusqlite/rusqlite/issues/1147).

```rust
use sql_split::split;
use rusqlite::{Connection, Result};

let conn = Connection::open_in_memory().expect("Can't open db in memory");
let statements = split("CREATE TABLE foo (bar: text); CREATE TABLE meep (moop: text)");
for s in statements {
    conn.execute(&s, []);
}
```
