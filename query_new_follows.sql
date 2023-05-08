SELECT
  'New follows since '
  ||
  (SELECT initial_end FROM meta)
  ||
  ' UTC:';

SELECT
  first_seen
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
