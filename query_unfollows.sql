SELECT
  'Unfollows since '
  ||
  (SELECT initial_end FROM meta)
  ||
  ' UTC:';

SELECT
  DATE(first_seen)
  ||
  ' to '
  ||
  last_seen
  ||
  ' https://twitter.com/i/user/'
  ||
  id
FROM
  follow
WHERE
  last_seen < (SELECT last_start FROM meta)
ORDER BY last_seen DESC;
