create table data_table (
  id text primary key not null,
  name text not null,
  data text not null
);

create table file_table (
  id text primary key not null,
  name text not null,
  -- This column stores binary large objects (BLOBs) with a medium size.
  -- In SQLite, other binary column types include:
  -- - BLOB: Stores binary data up to the maximum size of the database.
  -- - TINYBLOB: Stores binary data up to 255 bytes.
  -- - MEDIUMBLOB: Stores binary data up to 16,777,215 bytes.
  -- - LONGBLOB: Stores binary data up to 4,294,967,295 bytes.
  data blob not null
);