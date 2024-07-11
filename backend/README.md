# Gamehub – Backend app

### Running this application

To run this application, ... (todo)

### Configuration

This application gathers its configuration from environment variables. The following table describes the environment
variables used by this application:

| Name                                                                | Description                                                              | Example                                                      |
|---------------------------------------------------------------------|--------------------------------------------------------------------------|--------------------------------------------------------------|
| GAMEHUB__SERVER__HOST                                               | The server host of this application.                                     | 0.0.0.0, localhost                                           |
| GAMEHUB__SERVER__PORT                                               | The server port of this application.                                     | 1234, 8000                                                   |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_URI_TYPE              | The JWKS URI type of the Google ID token verifier.                       | Direct, AutoDiscover                                         |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_URI                   | The JWKS URI of the Google ID token verifier.                            | https://accounts.google.com/.well-known/openid-configuration |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_MAX_AGE               | The max age of entries of the Google ID token verifier's JWKS cache.     | 3600 (1 hour), None                                          |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__ISS                        | The valid issuers for the Google ID token verifier.                      | accounts.google.com, https://accounts.google.com             |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__AUD                        | The valid audience for the Google ID token verifier.                     | my_google_project_id, your_google_project_id                 |
| GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__ALLOW_UNSAFE_CONFIGURATION | Whether unsafe configuration of the Google ID token verifier is allowed. | true, false                                                  |

### Project considerations

This project has several considerations aimed at aligning the style, modularization, and other aspects of development.
These considerations are listed below:

#### General

1. Maximum test and documentation coverage

#### Modularization

1. Feature-based modularization
2. One `struct`/`enum`/`trait` per file
