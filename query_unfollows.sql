SELECT
  'Unfollows, starting at '
  ||
  (SELECT DATETIME(initial_end, 'localtime') FROM meta);

SELECT
  DATE(first_seen, 'localtime')
  ||
  ' to '
  ||
  DATETIME(last_seen, 'localtime')
  ||
  ' https://twitter.com/i/user/'
  ||
  id
FROM
  follow
WHERE
  last_seen < (SELECT last_start FROM meta)
ORDER BY last_seen DESC;
