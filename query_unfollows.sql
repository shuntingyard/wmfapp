SELECT
  'Unfollows, starting at '
  ||
  (SELECT DATE(initial_end, 'localtime') FROM meta);

SELECT
  DATE(first_seen, 'localtime')
  ||
  ' to '
  ||
  DATETIME(last_seen, 'localtime')
  ||
  '   https://twitter.com/i/user/'
  ||
  FORMAT('%-23s', id)
  ,
  last_handle_seen
  --,
  --last_name_seen
FROM
  follow
WHERE
  last_seen < (SELECT last_start FROM meta)
ORDER BY last_seen DESC;
