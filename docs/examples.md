# Examples

This document provides comprehensive examples of using the `dot` CLI tool.

## Basic Usage Examples

### Example 1: Setting up a new project

```bash
# Create a new project directory
mkdir my-awesome-project
cd my-awesome-project

# Initialize git
git init
git remote add origin git@github.com:myuser/my-awesome-project.git

# Initialize dot with multiple hidden directories
dot init .kiro .config .secrets

# Create some content
echo "# My Awesome Project" > README.md
echo '{"theme": "dark", "debug": true}' > .kiro/settings.json
echo 'api_key=dev_key_123' > .config/development.env
echo 'production_secret=super_secret' > .secrets/prod.env

# Add and commit everything
dot add .
dot commit -m "Initial project setup with configurations"
dot push
```

### Example 2: Working with an existing project

```bash
# Check if the current project has dot initialized
cd existing-project
dot status

# If not initialized, initialize it
dot init .kiro

# Add some hidden configuration
mkdir -p .kiro/workflows
echo 'build_command=cargo build --release' > .kiro/workflows/ci.conf

# Commit the new configuration
dot add .kiro/workflows/ci.conf
dot commit -m "Add CI workflow configuration"
dot push
```

### Example 3: Cloning a project with hidden repositories

```bash
# Clone a project that has dot configured
cd ~/projects
dot clone git@github.com:myuser/my-awesome-project.git

# The command above will:
# 1. Clone the main repository
# 2. Automatically discover and clone .kiro, .config, and .secrets repositories
# 3. Set up the local dot configuration

# Verify everything was cloned
cd my-awesome-project
ls -la
# Should show: README.md, .kiro/, .config/, .secrets/

dot status
# Should show status of all repositories
```

## Advanced Usage Examples

### Example 4: Non-atomic operations

Sometimes you want to continue even if some operations fail:

```bash
# Add files with non-atomic behavior
dot add . --no-atomic

# Commit with non-atomic behavior (useful when some repos have nothing to commit)
dot commit -m "Partial update" --no-atomic

# Push with non-atomic behavior
dot push --no-atomic
```

### Example 5: Working with only the main repository

```bash
# Check status of only the main repository
dot status --skip-hidden

# Add files only to the main repository
dot add README.md --skip-hidden

# Commit only to the main repository
dot commit -m "Update main documentation" --skip-hidden

# Push only the main repository
dot push --skip-hidden
```

### Example 6: Complex multi-directory setup

```bash
# Initialize with many hidden directories
dot init .kiro .config .secrets .cache .logs .temp

# Create structured content
mkdir -p .kiro/{workflows,settings,templates}
mkdir -p .config/{development,staging,production}
mkdir -p .secrets/{api-keys,certificates,tokens}

# Add configuration files
echo '{"env": "development"}' > .config/development/app.json
echo '{"env": "production"}' > .config/production/app.json
echo 'api_key=dev_123' > .secrets/api-keys/development.env
echo 'api_key=prod_xyz' > .secrets/api-keys/production.env

# Commit everything
dot add .
dot commit -m "Set up comprehensive configuration structure"
dot push
```

## Workflow Examples

### Example 7: Daily development workflow

```bash
# Start of day - check status
dot status

# Make changes to both main code and configuration
echo "console.log('new feature');" >> src/main.js
echo '{"new_feature": true}' > .kiro/feature-flags.json

# Stage and commit changes
dot add .
dot commit -m "Add new feature with configuration"

# Push everything
dot push

# Later, pull updates (standard git commands work on main repo)
git pull

# For hidden repos, you might need to pull them individually
# or use a script to pull all
```

### Example 8: Team collaboration workflow

```bash
# Team member A sets up the project
dot init .kiro .config
echo '{"team": "backend", "env": "shared"}' > .kiro/team-config.json
dot add .
dot commit -m "Add team configuration"
dot push

# Team member B clones and gets everything
dot clone git@github.com:team/project.git
cd project

# Team member B adds their own configuration
echo '{"user": "member-b", "role": "developer"}' > .kiro/user-config.json
dot add .kiro/user-config.json
dot commit -m "Add member B configuration"
dot push

# Team member A pulls updates to main repo and manually updates hidden repos
git pull
# For hidden repos, they would need to pull manually or use scripts
```

### Example 9: Configuration management across environments

```bash
# Set up environment-specific configurations
dot init .config

# Development environment
mkdir -p .config/dev
echo 'DEBUG=true' > .config/dev/.env
echo 'DB_HOST=localhost' >> .config/dev/.env

# Staging environment
mkdir -p .config/staging
echo 'DEBUG=false' > .config/staging/.env
echo 'DB_HOST=staging.db.company.com' >> .config/staging/.env

# Production environment (in secrets)
dot init .secrets
mkdir -p .secrets/prod
echo 'DB_PASSWORD=super_secret_password' > .secrets/prod/.env

# Commit all configurations
dot add .
dot commit -m "Add environment-specific configurations"
dot push
```

### Example 10: Backup and restore workflow

```bash
# Create a backup-friendly project structure
dot init .kiro .config .data

# Add important data that needs backup
echo '{"important": "data"}' > .data/critical.json
echo '{"backup": "daily"}' > .kiro/backup-config.json

# Commit and push (this serves as a backup)
dot add .
dot commit -m "Add critical data and backup configuration"
dot push

# Later, restore on a different machine
dot clone git@github.com:myuser/my-project.git
cd my-project
# All data including .data/ is automatically restored
```

## Error Handling Examples

### Example 11: Handling common errors

```bash
# Error: Organization not authorized
dot init .kiro
# Error: Organization 'my-org' not authorized in ~/.dot/dot.conf

# Solution: Add organization to config
echo '{"authorized_organizations": ["my-org"], "default_organization": "my-org"}' > ~/.dot/dot.conf

# Error: No git remote origin
mkdir new-project && cd new-project
dot init .kiro
# Error: Invalid git remote origin URL

# Solution: Set up git properly
git init
git remote add origin git@github.com:myuser/new-project.git
dot init .kiro

# Error: GitHub API issues
dot push
# Error: GitHub API error: Bad credentials

# Solution: Check GitHub token
echo $GITHUB_TOKEN
# If empty, set it:
export GITHUB_TOKEN="your_token_here"
```

### Example 12: Recovery from failed operations

```bash
# If an atomic operation fails, dot will automatically rollback
dot commit -m "This might fail"
# If it fails, all repositories are rolled back to previous state

# You can check what happened
dot status

# Fix the issue and try again
# For example, if there were unstaged changes:
dot add .
dot commit -m "Fixed commit with all changes staged"
```

## Integration Examples

### Example 13: Integration with CI/CD

```bash
# Set up CI configuration in hidden directory
dot init .kiro

# Create CI configuration
mkdir -p .kiro/ci
echo 'build: cargo build --release' > .kiro/ci/commands.yml
echo 'test: cargo test' >> .kiro/ci/commands.yml

# Create deployment secrets (in a separate secrets repo)
dot init .secrets
echo 'DEPLOY_KEY=secret_key_here' > .secrets/deploy.env

# Commit CI setup
dot add .
dot commit -m "Add CI/CD configuration and secrets"
dot push

# In your CI system, you would clone with dot:
# dot clone git@github.com:myuser/project.git
# This gets both the code and the CI configuration
```

### Example 14: Integration with development tools

```bash
# Set up tool-specific configurations
dot init .kiro .config

# IDE settings
mkdir -p .kiro/ide
echo '{"theme": "dark", "font_size": 14}' > .kiro/ide/vscode.json

# Development tools
mkdir -p .config/tools
echo 'format_on_save=true' > .config/tools/prettier.conf
echo 'auto_fix=true' > .config/tools/eslint.conf

# Database configuration
echo 'connection_string=postgresql://localhost/dev_db' > .config/database.conf

# Commit all tool configurations
dot add .
dot commit -m "Add development tool configurations"
dot push

# When team members clone, they get all the tool configurations
dot clone git@github.com:team/project.git
# Now everyone has the same IDE and tool settings
```

These examples demonstrate the flexibility and power of the `dot` CLI tool for managing complex project configurations while keeping sensitive or environment-specific files organized and version-controlled.