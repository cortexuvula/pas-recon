# Release Setup — One-Time Configuration

These steps are done once, by the repo owner, before the first release.

## 1. Generate the updater keypair

> Note: An updater keypair was already generated for this project and the
> public key is embedded in `crates/app/tauri.conf.json`. The private key is
> NOT in the repo. If you do not have the private key, regenerate a new pair
> following the steps below (and replace the pubkey in `tauri.conf.json`).

```bash
cargo tauri signer generate -w ~/.tauri/pas-recon-updater
```

This creates two files:
- `~/.tauri/pas-recon-updater` (private key — KEEP SECRET)
- `~/.tauri/pas-recon-updater.pub` (public key — embed in the app)

If running in a non-interactive environment (CI), use:

```bash
cargo tauri signer generate -w ~/.tauri/pas-recon-updater -p "<your password>" --ci -f
```

## 2. Embed the public key

The public key is already embedded in `crates/app/tauri.conf.json` under
`plugins.updater.pubkey`. If you regenerated the keypair, copy the contents of
the `.pub` file into that field.

## 3. Add secrets to the GitHub repo

Go to repo Settings → Secrets and variables → Actions. Add:

| Secret | Value |
|--------|-------|
| `TAURI_PRIVATE_KEY` | Contents of the private key file |
| `TAURI_KEY_PASSWORD` | The password you set during key generation (empty string if none) |

## 4. (Optional) Code signing

For silent auto-update (no OS prompts), you need:

### macOS
| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 |
| `APPLE_API_ISSUER` | App Store Connect API Issuer ID (UUID) |
| `APPLE_API_KEY` | App Store Connect API Key ID |
| `APPLE_API_KEY_CONTENT` | Full contents of the downloaded .p8 file |
| `APPLE_TEAM_ID` | Short team code (e.g. ABC123XYZ) |

Note: The app uses App Store Connect API key authentication for notarization, not the deprecated app-specific password method.

### Windows
| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Code signing certificate |

**Without these**, the app still builds and updates download, but the OS
will show "unidentified developer" warnings. Acceptable for initial release.

## 5. Create the GitHub repo and push

```bash
gh repo create pas-recon --source=. --push
```

The `endpoints` URL in `crates/app/tauri.conf.json` is already configured with the correct owner (`cortexuvula/pas-recon`).
  with your actual GitHub username/org.

## 6. Create the first release

```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions workflow builds all platforms and uploads installers +
the `latest.json` manifest to the release.

## VirusTotal scan

After the build matrix uploads installers to the release, a `virustotal-scan` job
downloads them, scans each on the **free public VirusTotal API**, and publishes:

- `VIRUSTOTAL-REPORT.md` as a release asset.
- A `## VirusTotal Scan` section appended to the release notes.

The job is `continue-on-error` and the script never exits non-zero — detections are
informational and never block a release.

### Required secret

Add a repository secret named **`VIRUSTOTAL_API_KEY`** with a free public API key from
<https://www.virustotal.com/gui/my-apikey>.

### Free-tier limits

- 32 MB upload cap. Larger installers fall back to a hash-only lookup; if the hash is
  unknown to VT, the file is recorded as "oversized" in the report.
- ~4 requests/minute. The scan job is single and rate-limits itself (>=16 s between calls),
  with exponential backoff on HTTP 429.
