# Svix Take-Home

## Todo
1. Add a migration system so the database does not need to be manually prepared.
2. Proper logging instead of println's
3. The "get all tasks" API route does not support filtering (ran out of time)
3. the "repeat" field is currently a boolean (and unused). The idea is to turn it into an optional TimeSpan so when the task is processed it can be "re-queued"