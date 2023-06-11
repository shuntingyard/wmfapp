SELECT
  'New follows, starting at '
  ||
  (SELECT DATE(initial_end, 'localtime') FROM meta);

SELECT
  '         from '
  ||
  DATETIME(first_seen, 'localtime')
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
  first_seen > (SELECT initial_end FROM meta)
  AND NOT
  last_seen < (SELECT last_start FROM meta)
ORDER BY first_seen DESC;
