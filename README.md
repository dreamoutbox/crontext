# crontext

natural-language schedule expressions parser and resolved into standard 5-field cron strings for the scheduler daemon.

## Supported Expressions

| Natural expression | Equivalent 5-field cron |
|---|---|
| `every minute` | `* * * * *` |
| `every 15 minutes` | `*/15 * * * *` |
| `every hour` | `0 * * * *` |
| `every 12 hours` | `0 */12 * * *` |
| `every day` | `0 0 * * *` |
| `every 1 day` | `0 0 * * *` |
| `every day at 03:30` | `30 3 * * *` |
| `every 12:00` | `0 12 * * *` |
| `every monday` | `0 0 * * 1` |
| `every friday at 18:00` | `0 18 * * 5` |
| `every mon, wed and fri at 06:30` | `30 6 * * 1,3,5` |
| `every weekday at 09:00` | `0 9 * * 1-5` |
| `every weekend at 10:00` | `0 10 * * 6,0` |
| `every month` | `0 0 1 * *` |
| `every month on the 1st at 03:00` | `0 3 1 * *` |
| `every 1st of february` | `0 0 1 2 *` |
| `every 15th of december` | `0 0 15 12 *` |
| `every 15th of dec at 09:00` | `0 9 15 12 *` |

## License

[MIT](LICENSE)
