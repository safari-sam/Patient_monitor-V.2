# GitHub Codespaces Setup Guide

## Quick Start (Development Mode)

The application **will run immediately** in Codespaces with default development credentials:

```bash
cd monitor
docker-compose up
```

✅ **Default credentials** (automatically used):
- `DB_PASSWORD`: `postgres`
- `JWT_SECRET`: `development_only_secret_replace_in_production`

⚠️ **WARNING**: These defaults are for DEVELOPMENT ONLY - never use in production!

## Production Setup (Secure Credentials)

For production or secure testing, configure Codespaces secrets:

### Step 1: Set Repository Secrets

1. Go to your GitHub repository
2. Navigate to: **Settings** → **Secrets and variables** → **Codespaces**
3. Click **New repository secret**
4. Add these secrets:

| Secret Name | Example Value | Description |
|-------------|---------------|-------------|
| `DB_PASSWORD` | `your_secure_db_password_123!` | PostgreSQL password |
| `JWT_SECRET` | `your_random_jwt_secret_key_xyz` | JWT signing key |

### Step 2: Generate Secure Secrets

**For JWT_SECRET** (generate a random 32-character string):
```bash
# Linux/macOS/Git Bash
openssl rand -base64 32

# PowerShell
-join ((48..57) + (65..90) + (97..122) | Get-Random -Count 32 | ForEach-Object {[char]$_})
```

**For DB_PASSWORD** (strong password):
- Minimum 16 characters
- Mix of uppercase, lowercase, numbers, symbols
- Example: `MyS3cur3P@ssw0rd!2026`

### Step 3: Restart Codespace

After setting secrets, **restart your Codespace** to load the new environment variables:
1. Stop the Codespace
2. Start it again
3. The secrets will be automatically injected

## Verification

### Check Environment Variables

```bash
# In Codespaces terminal
echo $DB_PASSWORD   # Should show your secret (or 'postgres' for dev mode)
echo $JWT_SECRET    # Should show your secret (or default for dev mode)
```

### Check Docker Containers

```bash
cd monitor
docker-compose up

# In another terminal:
docker-compose exec app env | grep -E 'DB_PASSWORD|JWT_SECRET'
```

### Test Authentication

```bash
# Should receive warning if using default JWT_SECRET:
# "JWT_SECRET not set in environment, using default (NOT SECURE FOR PRODUCTION!)"

# Check logs:
docker-compose logs app | grep JWT_SECRET
```

## Security Best Practices

### ✅ DO:
- Set Codespaces secrets for production/testing
- Use different secrets for each environment
- Rotate secrets regularly (every 90 days)
- Use strong, randomly generated passwords
- Keep secrets in GitHub Secrets (never commit to repo)

### ❌ DON'T:
- Commit `.env` files with real secrets
- Share secrets in chat/email
- Use default credentials in production
- Hardcode secrets in Dockerfile or docker-compose.yml
- Push secrets to GitHub repository

## Fallback Behavior

If secrets are **NOT set** in Codespaces:

| Variable | Fallback Value | Security Risk |
|----------|----------------|---------------|
| `DB_PASSWORD` | `postgres` | ⚠️ Medium (database access) |
| `JWT_SECRET` | `development_only_secret_replace_in_production` | 🔴 HIGH (token forging) |

**Recommendation**: Always set `JWT_SECRET` for any non-local testing!

## Troubleshooting

### Problem: "JWT_SECRET not set" warning in logs

**Solution**: 
1. Set the Codespaces secret (see Step 1 above)
2. Restart the Codespace
3. Verify with `echo $JWT_SECRET`

### Problem: Database connection refused

**Solution**:
```bash
# Check if database container is running
docker-compose ps

# Restart database
docker-compose restart db

# Check database logs
docker-compose logs db
```

### Problem: Secrets not loading

**Solution**:
1. Secrets are injected at Codespace **startup** only
2. **Rebuild** the Codespace:
   - Go to Codespaces dashboard
   - Click "..." → "Rebuild container"
3. Or create a **new Codespace**

### Problem: Using wrong secrets

**Solution**:
```bash
# List all environment variables
env | grep -E 'DB_PASSWORD|JWT_SECRET'

# Check docker-compose values
docker-compose config | grep -E 'DB_PASSWORD|JWT_SECRET'
```

## Local Development vs Codespaces

| Aspect | Local Development | GitHub Codespaces |
|--------|-------------------|-------------------|
| **Secrets** | `.env` file | GitHub Codespaces Secrets |
| **Setup** | Copy `.env.example` to `.env` | Set repository secrets |
| **Security** | Don't commit `.env` | Secrets encrypted by GitHub |
| **Fallback** | Uses defaults if `.env` missing | Uses defaults if secrets not set |

## Additional Resources

- [GitHub Codespaces Secrets Documentation](https://docs.github.com/en/codespaces/managing-your-codespaces/managing-encrypted-secrets-for-your-codespaces)
- [Environment Variables Best Practices](https://12factor.net/config)
- [Docker Compose Environment Variables](https://docs.docker.com/compose/environment-variables/)

---

**Status**: ✅ Ready for Codespaces (Development & Production)
**Last Updated**: January 2026
