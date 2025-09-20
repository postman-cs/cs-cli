#!/bin/bash

echo "=== GitHub Authentication Status ==="
echo

echo "1. Checking for GitHub token in keychain..."
TOKEN_OUTPUT=$(security find-generic-password -s "com.postman.cs-cli.github-token" -a "oauth-access-token" -w 2>/dev/null)
if [ $? -eq 0 ] && [ -n "$TOKEN_OUTPUT" ]; then
    echo "✅ GitHub token found in keychain (length: ${#TOKEN_OUTPUT})"
else
    echo "❌ No GitHub token found in keychain"
fi

echo
echo "2. Checking for gist configuration..."
CONFIG_FILE="$HOME/Library/Application Support/cs-cli/gist_config.json"
if [ -f "$CONFIG_FILE" ]; then
    echo "✅ Gist configuration file found"
    echo "   File: $CONFIG_FILE"
    echo "   Contents:"
    cat "$CONFIG_FILE" | jq . 2>/dev/null || cat "$CONFIG_FILE"
else
    echo "❌ No gist configuration file found"
fi

echo
echo "3. Testing GitHub API access..."
if command -v curl >/dev/null 2>&1; then
    if [ -n "$TOKEN_OUTPUT" ]; then
        USER_INFO=$(curl -s -H "Authorization: token $TOKEN_OUTPUT" https://api.github.com/user)
        if echo "$USER_INFO" | grep -q '"login"'; then
            USERNAME=$(echo "$USER_INFO" | jq -r '.login' 2>/dev/null || echo "unknown")
            echo "✅ GitHub API access working (user: $USERNAME)"
        else
            echo "❌ GitHub API access failed"
            echo "   Response: $USER_INFO"
        fi
    else
        echo "⚠️  Cannot test GitHub API - no token available"
    fi
else
    echo "⚠️  Cannot test GitHub API - curl not available"
fi

echo
echo "=== Summary ==="
if [ -n "$TOKEN_OUTPUT" ] && [ -f "$CONFIG_FILE" ]; then
    echo "✅ GitHub authentication appears to be fully configured"
    echo "   The login flow should skip OAuth and use existing credentials"
else
    echo "❌ GitHub authentication is not fully configured"
    echo "   OAuth flow will run to set up authentication"
fi