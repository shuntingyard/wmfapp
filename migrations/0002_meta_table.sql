-- http://www.databasesoup.com/2012/06/creating-table-with-exactly-one-row.html

CREATE TABLE meta (
    owner INTEGER NOT NULL,
    initial_end DATETIME,
    last_start DATETIME,
    last_end DATETIME,
    curr_start DATETIME
);

CREATE UNIQUE INDEX meta_one_row
ON meta ((owner IS NOT NULL));
