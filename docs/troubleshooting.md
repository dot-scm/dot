# Troubleshooting Guide

This guide helps you resolve common issues when using the `dot` CLI tool.

## Installation Issues

### Issue: `dot` command not found

**Symptoms:**
```bash
dot --version
# bash: dot: command not found
```

**Solutions:**

1. **Check if dot is installed:**
   ```bash
   which dot
   # If empty, dot is not installed
   ```

2. **Install dot:**
   ```bash
   git clone https://github.com/username/dot.git
   cd dot
   make install
   ```

3. **Check PATH:**
   ```bash
   echo $PATH
   # Make sure ~/.cargo/bin is in your PATH
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

4. **Verify installation:**
   ```bash
   make check-install
   ```

### Issue: Rust/Cargo not found during installation

**Symptoms:**
```bash
make install
# cargo: command not found
```

**Solutions:**

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Update Rust (if already installed):**
   ```bash
   rustup update
   ```

3. **Verify Rust installation:**
   ```bash
   rustc --version
   cargo --version
   ```

## Configuration Issues

### Issue: "Organization not authorized"

**Symptoms:**
```bash
dot init .kiro
# Error: Organization 'my-org' not authorized in ~/.dot/dot.conf
```

**Solutions:**

1. **Check current configuration:**
   ```bash
   cat ~/.dot/dot.conf
   ```

2. **Create or update configuration:**
   ```bash
   mkdir -p ~/.dot
   echo '{
     "authorized_organizations": ["my-org", "another-org"],
     "default_organization": "my-org"
   }' > ~/.dot/dot.conf
   ```

3. **Use make command to create config:**
   ```bash
   make create-config
   nano ~/.dot/dot.conf
   ```

4. **Verify configuration:**
   ```bash
   make show-config
   ```

### Issue: "Failed to load configuration"

**Symptoms:**
```bash
dot status
# Failed to load configuration: Invalid JSON format
```

**Solutions:**

1. **Check JSON syntax:**
   ```bash
   cat ~/.dot/dot.conf | python -m json.tool
   # or
   cat ~/.dot/dot.conf | jq .
   ```

2. **Fix common JSON errors:**
   - Missing commas between array elements
   - Trailing commas
   - Unescaped quotes
   - Missing closing braces

3. **Recreate configuration:**
   ```bash
   rm ~/.dot/dot.conf
   make create-config
   ```

## GitHub Integration Issues

### Issue: "GitHub API error: Bad credentials"

**Symptoms:**
```bash
dot push
# Error: GitHub API error: Bad credentials
```

**Solutions:**

1. **Check if GitHub token is set:**
   ```bash
   echo $GITHUB_TOKEN
   # Should show your token
   ```

2. **Set GitHub token:**
   ```bash
   export GITHUB_TOKEN="your_github_token_here"
   
   # Make it permanent
   echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.bashrc
   # or for zsh
   echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.zshrc
   ```

3. **Verify token permissions:**
   - Go to GitHub Settings > Developer settings > Personal access tokens
   - Ensure your token has `repo` permissions
   - Check if token is expired

4. **Test token manually:**
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
   ```

### Issue: "Repository already exists"

**Symptoms:**
```bash
dot init .kiro
# Error: Repository with key 'github.com/user/project/.kiro' already exists
```

**Solutions:**

1. **Check if project is already initialized:**
   ```bash
   dot status
   ```

2. **If you want to reinitialize, first clean up:**
   ```bash
   # This is destructive - backup first!
   # Remove from index (manual process)
   # Delete the hidden repository from GitHub
   ```

3. **Use a different directory name:**
   ```bash
   dot init .kiro-new
   ```

### Issue: "Failed to create repository"

**Symptoms:**
```bash
dot init .kiro
# Error: Failed to create repository: API rate limit exceeded
```

**Solutions:**

1. **Wait for rate limit reset:**
   ```bash
   # GitHub API rate limits reset every hour
   # Wait and try again
   ```

2. **Check rate limit status:**
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit
   ```

3. **Use authenticated requests (should have higher limits):**
   - Ensure GITHUB_TOKEN is properly set
   - Use a personal access token, not basic auth

## Git Operation Issues

### Issue: "git is not installed or not in PATH"

**Symptoms:**
```bash
dot status
# Error: git is not installed or not in PATH
```

**Solutions:**

1. **Install Git:**
   ```bash
   # macOS
   brew install git
   
   # Ubuntu/Debian
   sudo apt-get install git
   
   # CentOS/RHEL
   sudo yum install git
   ```

2. **Check Git installation:**
   ```bash
   git --version
   which git
   ```

3. **Add Git to PATH:**
   ```bash
   export PATH="/usr/local/bin:$PATH"
   ```

### Issue: "Invalid git remote origin URL"

**Symptoms:**
```bash
dot init .kiro
# Error: Invalid git remote origin URL
```

**Solutions:**

1. **Check current remote:**
   ```bash
   git remote -v
   ```

2. **Add remote origin:**
   ```bash
   git remote add origin git@github.com:username/repository.git
   # or
   git remote add origin https://github.com/username/repository.git
   ```

3. **Fix existing remote:**
   ```bash
   git remote set-url origin git@github.com:username/repository.git
   ```

4. **Initialize git if needed:**
   ```bash
   git init
   git remote add origin git@github.com:username/repository.git
   ```

### Issue: "Working directory is not clean"

**Symptoms:**
```bash
dot commit -m "test"
# Error: Working directory is not clean
```

**Solutions:**

1. **Check status:**
   ```bash
   dot status
   git status
   ```

2. **Add unstaged changes:**
   ```bash
   dot add .
   dot commit -m "test"
   ```

3. **Stash changes if needed:**
   ```bash
   git stash
   dot commit -m "test"
   git stash pop
   ```

## Atomic Operation Issues

### Issue: "Atomic operation failed, rolling back"

**Symptoms:**
```bash
dot commit -m "test"
# Error: Atomic operation failed, rolling back changes
```

**Solutions:**

1. **Check what failed:**
   ```bash
   dot status
   # Look for error messages in each repository
   ```

2. **Use non-atomic mode:**
   ```bash
   dot commit -m "test" --no-atomic
   ```

3. **Fix issues in individual repositories:**
   ```bash
   # Check each hidden directory
   cd .kiro
   git status
   git add .
   cd ..
   
   # Try again
   dot commit -m "test"
   ```

4. **Skip hidden repositories temporarily:**
   ```bash
   dot commit -m "test" --skip-hidden
   ```

### Issue: "Rollback failed"

**Symptoms:**
```bash
dot push
# Error: Push failed, rollback also failed
```

**Solutions:**

1. **Manual cleanup required:**
   ```bash
   # Check status of all repositories
   dot status
   
   # Manually fix each repository
   git status
   cd .kiro && git status && cd ..
   ```

2. **Reset to last known good state:**
   ```bash
   git reset --hard HEAD~1
   cd .kiro && git reset --hard HEAD~1 && cd ..
   ```

3. **Contact support if data is critical**

## Performance Issues

### Issue: "Operations are very slow"

**Solutions:**

1. **Check network connectivity:**
   ```bash
   ping github.com
   ```

2. **Use SSH instead of HTTPS:**
   ```bash
   git remote set-url origin git@github.com:username/repository.git
   ```

3. **Check repository sizes:**
   ```bash
   du -sh .git
   cd .kiro && du -sh .git && cd ..
   ```

4. **Clean up repository history if needed:**
   ```bash
   git gc --aggressive
   ```

## Index Repository Issues

### Issue: "Failed to access index repository"

**Symptoms:**
```bash
dot init .kiro
# Error: Failed to access index repository
```

**Solutions:**

1. **Check if .index repository exists:**
   - Go to your GitHub organization
   - Look for `.index` repository

2. **Create index repository manually:**
   ```bash
   # Go to GitHub and create a new repository named ".index"
   # Make it private
   # Initialize with README
   ```

3. **Check organization permissions:**
   - Ensure your GitHub token has access to the organization
   - Verify you have admin rights to create repositories

4. **Clear local index cache:**
   ```bash
   rm -rf ~/.dot/.index
   # Try the operation again
   ```

## Debug Mode

### Enable verbose logging

```bash
# Set environment variable for detailed logs
export RUST_LOG=debug
dot status

# Or for even more detail
export RUST_LOG=trace
dot init .kiro
```

### Check system information

```bash
# Check dot version
dot --version

# Check system info
uname -a
git --version
cargo --version

# Check configuration
make show-config
```

## Getting Help

If you're still experiencing issues:

1. **Check the documentation:**
   - [README](../README.md)
   - [Examples](examples.md)

2. **Search existing issues:**
   - [GitHub Issues](https://github.com/username/dot/issues)

3. **Create a new issue with:**
   - Your operating system
   - `dot --version` output
   - Complete error message
   - Steps to reproduce
   - Your configuration (remove sensitive data)

4. **Include debug information:**
   ```bash
   # Run with debug logging
   RUST_LOG=debug dot status 2>&1 | tee debug.log
   # Attach debug.log to your issue
   ```

## Common Command Reference

```bash
# Check installation
make check-install

# Show configuration
make show-config

# Create default configuration
make create-config

# Test basic functionality
dot --version
dot --help

# Check status (safe operation)
dot status

# Reset configuration
rm ~/.dot/dot.conf
make create-config
```