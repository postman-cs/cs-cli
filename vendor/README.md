# Vendored Dependencies

This directory contains vendored dependencies that are critical for the CS-CLI project.

## impit

**Source**: https://github.com/apify/impit.git
**Commit**: f47c814 (docs(ci): fix missing GitHub Pages upload)
**Purpose**: HTTP client with TLS fingerprinting and browser impersonation capabilities

### Why Vendored?

The impit library is critical for bypassing anti-bot detection on enterprise platforms. We vendor it locally to:

1. **Ensure availability**: Protect against the upstream repository being deleted or made private
2. **Version stability**: Lock to a specific known-working version
3. **Offline development**: Enable development without network access to GitHub
4. **Security**: Review and audit the exact code being used

### Updating impit

If you need to update the vendored impit:

```bash
cd vendor/impit
git pull origin master
# Test thoroughly before committing
```

### Restoring if Missing

If the vendor directory is missing, restore it with:

```bash
mkdir -p vendor
cd vendor
git clone https://github.com/apify/impit.git
cd impit
git checkout f47c814  # Or latest stable commit
```

Then ensure `Cargo.toml` points to the local path:
```toml
impit = { path = "vendor/impit/impit" }
```