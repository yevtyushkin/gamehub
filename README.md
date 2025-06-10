# gamehub

[![codecov](https://codecov.io/gh/yevtyushkin/gamehub/graph/badge.svg?token=HM3QY3R0U0)](https://codecov.io/gh/yevtyushkin/gamehub)

### Configuration

The following environment variables are used:

| Name                          | Description                                                                                                                        | Example           |
|-------------------------------|------------------------------------------------------------------------------------------------------------------------------------|-------------------|
| `SERVER__HOST`                | Server listening host                                                                                                              | 127.0.0.1         |
| `SERVER__PORT`                | Server listening port                                                                                                              | 8080              |
| `POSTGRES__HOST`              | Postgres host                                                                                                                      | 127.0.0.1         |
| `POSTGRES__PORT`              | Postgres port                                                                                                                      | 5432              |
| `POSTGRES__USERNAME`          | Postgres username                                                                                                                  | postgres_username |
| `POSTGRES__PASSWORD`          | Postgres password                                                                                                                  | postgres_password |
| `POSTGRES__DATABASE`          | Postgres database name                                                                                                             | postgres_database |
| `JWT__SECRET`                 | JWT secret                                                                                                                         | s3cr3t            |
| `JWT__TTL`                    | JWT TTL                                                                                                                            | 1h                |
| `GOOGLE_ID_TOKEN_VERIFIER__*` | Google ID token verifier configuration, see [id_token_verifier](https://github.com/yevtyushkin/id_token_verifier) for more details |                   |
