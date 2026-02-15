# Authentication Examples

This directory contains example authentication configuration files for Hazler.

## Overview

Hazler supports multiple authentication methods for crawling protected web applications and APIs:

- **Basic Authentication** - HTTP Basic Auth with username/password
- **Bearer Token** - JWT or other bearer tokens
- **Cookie-based** - Session cookies
- **Custom Headers** - API keys or custom authentication headers
- **API Key** - Flexible API key placement (header/query/cookie)
- **OAuth 2.0** - OAuth2 access tokens with refresh support

## Usage

### Command-Line Arguments

You can specify authentication directly via CLI flags:

```bash
# Basic Auth
hazler https://api.example.com --auth-basic "username:password"

# Bearer Token
hazler https://api.example.com --auth-bearer "eyJhbGc..."

# Cookie
hazler https://app.example.com --auth-cookie "session=abc123"

# Custom Header
hazler https://api.example.com --auth-header "X-API-Key:secret123"

# API Key (in header)
hazler https://api.example.com --auth-apikey "sk-123456" --auth-apikey-name "X-API-Key"

# API Key (in query)
hazler https://api.example.com --auth-apikey "sk-123456" --auth-apikey-location query --auth-apikey-name "api_key"

# OAuth2
hazler https://api.example.com --auth-oauth "ya29.a0AfH6SMBx..."
```

### Configuration Files

For complex authentication scenarios or to avoid exposing credentials in shell history, use JSON configuration files:

```bash
# Load from file
hazler https://api.example.com --auth-file examples/auth-basic.json
```

## Form-Based Login

For applications that use form-based login:

```bash
hazler https://app.example.com \
  --auth-form-url "https://app.example.com/login" \
  --auth-form-username "admin" \
  --auth-form-password "secret123" \
  --auth-cookie "session=initial-session"
```

The crawler will:
1. Submit credentials to the login form
2. Extract and store session cookies automatically
3. Use those cookies for subsequent requests

## Configuration File Format

### Basic Authentication

```json
{
  "method": {
    "Basic": {
      "username": "admin",
      "password": "secret123"
    }
  },
  "session": {
    "enabled": true,
    "refresh_interval": 0,
    "refresh_url": null,
    "refresh_credentials": null
  },
  "form_auth": null
}
```

### Bearer Token

```json
{
  "method": {
    "Bearer": {
      "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    }
  },
  "session": {
    "enabled": true,
    "refresh_interval": 0,
    "refresh_url": null,
    "refresh_credentials": null
  },
  "form_auth": null
}
```

### Cookie-Based

```json
{
  "method": {
    "Cookie": {
      "cookies": {
        "session": "abc123def456",
        "user_id": "12345"
      }
    }
  },
  "session": {
    "enabled": true,
    "refresh_interval": 0,
    "refresh_url": null,
    "refresh_credentials": null
  },
  "form_auth": null
}
```

### API Key

```json
{
  "method": {
    "ApiKey": {
      "key": "sk-1234567890abcdef",
      "location": "Header",
      "name": "X-API-Key"
    }
  },
  "session": {
    "enabled": true,
    "refresh_interval": 0,
    "refresh_url": null,
    "refresh_credentials": null
  },
  "form_auth": null
}
```

Available locations: `"Header"`, `"Query"`, `"Cookie"`

### OAuth 2.0

```json
{
  "method": {
    "OAuth2": {
      "access_token": "ya29.a0AfH6SMBx...",
      "token_type": "Bearer",
      "refresh_token": "1//0gKl2...",
      "expires_in": 3600
    }
  },
  "session": {
    "enabled": true,
    "refresh_interval": 3300,
    "refresh_url": "https://oauth2.googleapis.com/token",
    "refresh_credentials": null
  },
  "form_auth": null
}
```

## Security Best Practices

1. **Never commit credentials to version control** - Use `.gitignore` for auth config files
2. **Use environment variables** - For sensitive values in CI/CD pipelines
3. **Rotate credentials regularly** - Especially after security assessments
4. **Use read-only tokens** - When possible, use tokens with minimal required permissions
5. **Monitor authentication logs** - Watch for failed authentication attempts

## Session Management

Hazler automatically manages sessions through:

- **Cookie Jar** - Cookies are automatically stored and sent with subsequent requests
- **Token Refresh** - OAuth2 tokens can be refreshed automatically (when refresh_url is configured)
- **Session Persistence** - Sessions remain active throughout the crawl

## Multi-User Crawling

To compare privilege levels (BOLA/IDOR detection):

```bash
# Crawl as user 1
hazler https://app.example.com --auth-basic "user1:pass1" -o user1.json

# Crawl as user 2
hazler https://app.example.com --auth-basic "user2:pass2" -o user2.json

# Compare results to find privilege escalation issues
diff user1.json user2.json
```

## Proxy with Authentication

Combine authentication with proxy support:

```bash
hazler https://api.example.com \
  --auth-bearer "token123" \
  --proxy "http://proxy.example.com:8080"
```

## Common Use Cases

### REST API Testing

```bash
hazler https://api.example.com/v1 \
  --auth-bearer "$(cat token.txt)" \
  --aggressive \
  -o json
```

### Web Application Crawling

```bash
hazler https://app.example.com \
  --auth-form-url "https://app.example.com/login" \
  --auth-form-username "$USERNAME" \
  --auth-form-password "$PASSWORD" \
  --browser \
  --max-depth 5
```

### GraphQL API Discovery

```bash
hazler https://api.example.com/graphql \
  --auth-apikey "$API_KEY" \
  --graphql-introspect \
  -o json
```

## Troubleshooting

### Authentication Not Working

1. Check credentials are correct
2. Verify the authentication method matches the API/application
3. Check for CSRF tokens or additional headers required
4. Use `--verbose` flag to see debug logs (credentials are sanitized)

### Session Expiring

1. Check token/session expiration time
2. Configure `refresh_interval` in config file
3. Use form-based login for applications with session refresh

### Rate Limiting

Combine with rate limiting flags:

```bash
hazler https://api.example.com \
  --auth-bearer "token" \
  --rate-limit 10 \
  --circuit-breaker
```

## Examples

See the example JSON files in this directory:

- `auth-basic.json` - HTTP Basic Authentication
- `auth-bearer.json` - Bearer Token (JWT)
- `auth-cookie.json` - Cookie-based Authentication
- `auth-apikey.json` - API Key Authentication
- `auth-oauth2.json` - OAuth 2.0 with Refresh

## Contributing

Found an authentication method not supported? Open an issue or pull request!
