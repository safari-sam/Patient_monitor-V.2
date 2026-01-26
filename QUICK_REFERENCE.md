# Quick Reference - CI/CD & Development Tools

## 🚀 Setup (One-Time)

### Install Git Hooks
```bash
# Linux/Mac
./setup-hooks.sh

# Windows
.\setup-hooks.ps1
```

## 📝 Daily Development

### Before Starting Work
```bash
cd monitor
docker-compose up -d     # Start database and ML service
```

### Code → Test → Commit
```bash
cd monitor/backend

# 1. Format code
cargo fmt

# 2. Check for issues
cargo clippy

# 3. Run tests
cargo test

# 4. Commit (hooks run automatically)
git add .
git commit -m "feat: your message"
```

## 🔍 Common Commands

### Code Quality
```bash
cargo fmt              # Auto-format code
cargo fmt --check      # Check without changing
cargo clippy           # Run linter
cargo clippy --fix     # Auto-fix issues
```

### Testing
```bash
cargo test                    # All tests
cargo test --lib              # Unit tests only
cargo test --test api_tests   # Integration tests
cargo test test_name          # Specific test
```

### Building
```bash
cargo build           # Debug build
cargo build --release # Optimized build
cargo run             # Build and run
```

### Docker
```bash
cd monitor
docker-compose up -d          # Start all services
docker-compose down           # Stop all services
docker-compose logs app       # View backend logs
docker-compose logs ml-service # View ML service logs
docker-compose ps             # Check status
```

## 🔧 Troubleshooting

### Pre-commit Hook Fails
```bash
# Fix formatting
cargo fmt

# Fix linter issues
cargo clippy --fix

# Run tests to see failures
cargo test

# Skip hooks (NOT RECOMMENDED)
git commit --no-verify
```

### CI/CD Pipeline Fails
1. Check GitHub Actions tab
2. Review failed job logs
3. Run same commands locally:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

### Docker Issues
```bash
# Rebuild containers
docker-compose down
docker-compose build --no-cache
docker-compose up -d

# Clear all Docker data (CAUTION)
docker system prune -a --volumes
```

## 📚 File Locations

```
.
├── .github/workflows/ci.yml      # CI/CD pipeline
├── hooks/pre-commit              # Git pre-commit hook
├── monitor/
│   ├── backend/
│   │   ├── rustfmt.toml         # Format config
│   │   ├── clippy.toml          # Linter config
│   │   └── src/
│   └── docker-compose.yml
├── setup-hooks.sh                # Bash setup script
├── setup-hooks.ps1               # PowerShell setup script
└── CONTRIBUTING.md               # Full guidelines
```

## 🎯 Quick Checks Before Push

✅ Code formatted: `cargo fmt --check`  
✅ No lint warnings: `cargo clippy -- -D warnings`  
✅ Tests pass: `cargo test`  
✅ No secrets in code: `git grep -E "password.*=.*\""`  
✅ Docker builds: `docker-compose build`

## 🔄 CI/CD Pipeline Jobs

When you push to GitHub, these run automatically:

1. **🔍 Lint & Format** - Code style check
2. **🧪 Run Tests** - All tests with PostgreSQL
3. **🔨 Build** - Release compilation
4. **🐳 Docker Build** - Container testing
5. **🤖 ML Tests** - ML service validation
6. **🔒 Security Scan** - Vulnerability check

## 💡 Tips

- **Format on save**: Configure your IDE to run `cargo fmt` automatically
- **Watch mode**: Use `cargo watch -x test` for continuous testing
- **Fast builds**: Use `cargo check` instead of `cargo build` during development
- **Parallel tests**: Tests run in parallel by default (use `--test-threads=1` to disable)

## 🆘 Need Help?

- Read CONTRIBUTING.md for detailed guidelines
- Check existing GitHub Issues
- Review code comments and documentation
