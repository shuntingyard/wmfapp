SELECT
  'New follows, starting at '
  ||
  (SELECT DATETIME(initial_end, 'localtime') FROM meta);

SELECT
  'since '
  ||
  DATETIME(first_seen, 'localtime')
  ||
  ' https://twitter.com/i/user/'
  ||
  id
FROM
  follow
WHERE
  first_seen > (SELECT initial_end FROM meta)
  AND NOT
  last_seen < (SELECT last_start FROM meta)
ORDER BY first_seen DESC;
