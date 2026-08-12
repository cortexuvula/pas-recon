# Windows Code Signing Setup (Azure Trusted Signing)

This documents the Azure-side setup for signing Windows installers in the release
workflow. The CI wiring lives in `.github/workflows/release.yml` (Windows leg); this
guide covers the manual Azure enrollment that produces the GitHub secrets.

Signing uses **Azure Trusted Signing** (now branded **Azure Artifact Signing**) —
Microsoft's cloud code-signing service. No hardware token; keys live in Azure.

## Prerequisites

- An **Azure subscription** (free tier works).
- For **individual** developer accounts, you must be in the **US or Canada**.
  (Organizations have broader availability.)
- ~1–2 business days for identity validation.

## Azure setup

1. **Register for Artifact Signing.** In the Azure Portal, subscribe to the
   "Trusted Signing" / "Artifact Signing" provider and complete **identity validation**
   (individual: passport or driver's license). This is the multi-day wait.
2. **Create an Artifact Signing account** in a supported region (e.g. `Canada Central`
   or `East US`). Note the **endpoint** for your region, e.g.:
   - East US: `https://eus.codesigning.azure.net`
   - West US 2: `https://wus2.codesigning.azure.net`
   - North Europe: `https://neu.codesigning.azure.net`

   The endpoint **must match the account region exactly**, or signing returns 403.
3. **Create a certificate profile** (public trust) inside the account.
4. **Register a Microsoft Entra ID app** (Portal → Microsoft Entra ID → App registrations
   → New registration). Under **Certificates & secrets**, create a **client secret** and
   copy the secret **Value** immediately (it's hidden once you leave the page).
5. **Grant the app signing access.** On the Artifact Signing account → **Access control
   (IAM)** → Add role assignment → role **"Artifact Signing Certificate Profile Signer"**,
   assign access to "User, group, or service principal", then **search for your app by
   name** (apps do not appear in the picker until you search).

## GitHub repository secrets

Add these (Settings → Secrets and actions → New repository secret):

| Secret | Value |
|---|---|
| `AZURE_TENANT_ID` | Entra **Tenant ID** (not the subscription ID) |
| `AZURE_CLIENT_ID` | App registration **Application (client) ID** (not the Object ID) |
| `AZURE_CLIENT_SECRET` | The client secret's **Value** (NOT its Secret ID) |
| `AZURE_ENDPOINT` | Region endpoint, e.g. `https://eus.codesigning.azure.net` |
| `AZURE_ACCOUNT` | Artifact Signing **account name** |
| `AZURE_PROFILE` | Certificate **profile name** |

> The single most common failure is putting the secret **ID** (a UUID) in
> `AZURE_CLIENT_SECRET` instead of the secret **Value**. They are different strings.

## How CI uses them

On the `windows-latest` leg of `release.yml`:
1. Installs the .NET `sign` tool (`dotnet tool install -g --prerelease sign`).
2. Generates a merge config (`crates/app/signing-config.json`) with a
   `bundle.windows.signCommand` that invokes `sign code artifact-signing ... %1`.
3. `tauri-action` builds with `--config signing-config.json`, so Tauri signs each
   binary and the installer **in-build** before the release upload.
4. Authenticates to Azure via `AZURE_TENANT_ID` / `AZURE_CLIENT_ID` /
   `AZURE_CLIENT_SECRET`.

The `endpoint` / `account` / `profile` are not secret — they're read from secrets only
to keep them out of the repo.

## Verifying a signed release

After a release whose Windows leg succeeded:
- `VIRUSTOTAL-REPORT.md` should show the Microsoft `Wacatac.B!ml` detection cleared
  (after a VirusTotal re-scan).
- The release's `setup.exe` / `.msi` show a digital signature in the file properties
  (Digital Signatures tab), publisher = your validated identity.

## SmartScreen reputation

Azure Trusted Signing is Microsoft-trusted, but **reputation builds gradually**. The
first signed release(s) may still trigger the "Windows protected your PC" SmartScreen
warning until reputation accrues over a few releases. This is expected and not a
configuration error. (An EV certificate would give instant reputation, at ~4–7× the
cost + hardware-token/HSM handling — not used here.)
