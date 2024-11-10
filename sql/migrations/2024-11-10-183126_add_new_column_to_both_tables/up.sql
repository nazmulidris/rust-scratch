-- Add a new column created_at to data_table. This can't be current_timestamp because
-- SQLite doesn't support that. The default value must be a constant.
alter table
  data_table
add
  column created_at timestamp not null default '1900-01-01 12:12:12';

-- Add a new column created_at to file_table. This can't be current_timestamp because
-- SQLite doesn't support that. The default value must be a constant.
alter table
  file_table
add
  column created_at timestamp not null default '1900-01-01 12:12:12';

-- Update the created_at column in data_table if needed (it is needed if the row's date is
-- hard coded to '1900-01-01 12:12:12'.
update
  data_table
set
  created_at = current_timestamp
where
  created_at is '1900-01-01 12:12:12';