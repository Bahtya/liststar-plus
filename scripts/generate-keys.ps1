# Listory Plus v2 Signing Key Generation Script
# This script generates a new signing key pair for Tauri auto-updates.

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   Tauri Signing Key Generation" -ForegroundColor Cyan
Write-Host "====================================================`n" -ForegroundColor Cyan

Write-Host "Generating keys using 'npx tauri signer generate'..." -ForegroundColor Gray
npx tauri signer generate

Write-Host "`n====================================================" -ForegroundColor Cyan
Write-Host "   GitHub Secrets Configuration Guide" -ForegroundColor Yellow
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "To enable auto-updates and signed releases in GitHub Actions,"
Write-Host "please add the following secrets to your repository:"
Write-Host ""
Write-Host "1. TAURI_SIGNING_PRIVATE_KEY" -ForegroundColor Green
Write-Host "   - Value: The 'Private key' printed above (starts with 'dW50cnVzdGVk...')"
Write-Host ""
Write-Host "2. TAURI_SIGNING_PASSWORD" -ForegroundColor Green
Write-Host "   - Value: The password you entered during generation (if any)."
Write-Host "   - Note: If you didn't enter a password, you can leave this empty or set a dummy value."
Write-Host ""
Write-Host "3. TAURI_SET_PUBKEY_HERE (Optional for local testing)" -ForegroundColor Green
Write-Host "   - Update 'pubkey' in 'src-tauri/tauri.conf.json' with the 'Public key' printed above."
Write-Host "====================================================" -ForegroundColor Cyan
