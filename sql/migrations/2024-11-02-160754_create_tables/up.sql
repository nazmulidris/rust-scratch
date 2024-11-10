create table if not exists data_table (
  id text primary key not null,
  name text not null,
  data text not null
);

create table if not exists file_table (
  id text primary key not null,
  name text not null,
  data text not null
);