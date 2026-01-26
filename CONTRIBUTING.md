# Contributing to Smart Patient Room Monitor

Thank you for your interest in contributing! This document provides guidelines for contributing to this project.

## 🚀 Quick Start

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/smart-patient-monitor.git
   cd smart-patient-monitor
   ```

2. **Install Git hooks** (one-time setup)
   ```bash
   # Linux/Mac
   chmod +x setup-hooks.sh
   ./setup-hooks.sh
   
   # Windows
   .\setup-hooks.ps1
   ```

3. **Set up development environment**
   ```bash
   cd monitor
   docker-compose up -d  # Start PostgreSQL and ML service
   ```

## 🔧 Development Setup

### Prerequisites
- Rust 1.70+ (`rustup install stable`)
- Docker & Docker Compose
- Git
- Python 3.11+ (for ML service development)

### Environment Variables
Copy `.env.example` to `.env` and configure:
```bash
cp monitor/.env.example monitor/.env
```

Required secrets:
- `JWT_SECRET` - Random string for JWT signing
- `DB_PASSWORD` - PostgreSQL password

## 📝 Code Standards

### Rust Code Style
- **Formatting**: Use `cargo fmt` (runs automatically in pre-commit hook)
- **Linting**: Fix all `cargo clippy` warnings
- **Line width**: 100 characters max
- **Imports**: Group by `std`, external crates, internal modules

### Testing Requirements
- **Unit tests**: Required for all new functions
- **Integration tests**: Required for new API endpoints
- **Test coverage**: Aim for >80% coverage
- **FHIR compliance**: Validate all FHIR structures

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add sleep pattern analysis endpoint
fix: correct fall detection threshold
docs: update API documentation
test: add tests for ML client
refactor: simplify FHIR transformation logic
```

## 🔄 Workflow

### 1. Create a Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 2. Make Changes
Write code following the style guidelines above.

### 3. Test Locally
```bash
cd monitor/backend

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Build
cargo build --release
```

### 4. Commit Changes
```bash
git add .
git commit -m "feat: your descriptive message"
```

The pre-commit hook will automatically:
- ✅ Check formatting
- ✅ Run Clippy
- ✅ Run unit tests
- ✅ Check for secrets

### 5. Push and Create PR
```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub. The CI/CD pipeline will:
- Run all tests
- Build Docker images
- Perform security scans
- Check code quality

## 🧪 Testing

### Run All Tests
```bash
cd monitor/backend
cargo test
```

### Run Specific Tests
```bash
cargo test test_name           # Run specific test
cargo test --lib               # Unit tests only
cargo test --test api_tests    # Specific integration test
```

### Test Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## 🔒 Security

### Never Commit:
- ❌ Passwords or API keys
- ❌ `.env` files (use `.env.example`)
- ❌ Private keys or certificates
- ❌ Patient data or PHI

### Always:
- ✅ Use environment variables for secrets
- ✅ Validate all user inputs
- ✅ Use parameterized SQL queries
- ✅ Follow OWASP security guidelines

## 📋 Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code follows Rust style guide (`cargo fmt`)
- [ ] All Clippy warnings resolved (`cargo clippy`)
- [ ] Tests pass locally (`cargo test`)
- [ ] New tests added for new features
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (if applicable)
- [ ] No hardcoded secrets
- [ ] Commit messages follow conventions
- [ ] PR description explains changes clearly

## 🏥 FHIR Compliance

When working with FHIR resources:

1. **Use standard codes**:
   - LOINC for observations
   - SNOMED CT for clinical terms
   - HL7 terminology for categories

2. **Validate resources**:
   ```bash
   # Use FHIR validator
   java -jar validator_cli.jar observation.json
   ```

3. **Follow FHIR R4 spec**: https://hl7.org/fhir/R4/

## 🐛 Reporting Bugs

Create an issue with:
- **Title**: Clear, descriptive summary
- **Description**: Steps to reproduce
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, Rust version, Docker version
- **Logs**: Relevant error messages

## 💡 Suggesting Features

Create an issue with:
- **Use case**: Clinical scenario or problem
- **Proposed solution**: How it should work
- **Alternatives**: Other approaches considered
- **FHIR impact**: How it affects FHIR compliance

## 📞 Getting Help

- **Documentation**: Check README.md and inline code comments
- **Issues**: Search existing issues before creating new ones
- **Discussions**: Use GitHub Discussions for questions

## 🎯 Project Goals

Keep in mind the project's core objectives:
1. **Clinical accuracy**: Reliable detection and monitoring
2. **FHIR compliance**: Interoperability with healthcare systems
3. **Performance**: Real-time processing capabilities
4. **Security**: HIPAA-ready security practices
5. **Usability**: Clear dashboards and alerts

## 📜 License

By contributing, you agree that your contributions will be licensed under the project's license.

---

Thank you for contributing to improve patient safety and care! 🏥
