# Google Calendar setup

The Calendar tab needs OAuth credentials to talk to Google. You only do this once per development machine (and once per release in CI).

## 1. Create a Google Cloud project

Go to <https://console.cloud.google.com> and either pick an existing project or create a new one. The name is just for you.

## 2. Enable the Calendar API

In the project, go to **APIs & Services → Library**, search "Google Calendar API", and click **Enable**.

## 3. Configure the OAuth consent screen

**APIs & Services → OAuth consent screen**:

- **User Type:** External
- **App name:** Postik (or whatever you want — only you see it)
- **User support email:** your email
- **Developer contact:** your email
- **Scopes:** click **Add or remove scopes** and add:
  - `https://www.googleapis.com/auth/calendar.readonly`
  - `https://www.googleapis.com/auth/userinfo.email`
- **Test users:** add the Google account(s) you'll connect to Postik (Google rejects the OAuth flow for non-test-users while the app is in "Testing" status — you do not need to publish).

## 4. Create OAuth credentials

**APIs & Services → Credentials → Create credentials → OAuth client ID**:

- **Application type:** Desktop app
- **Name:** Postik desktop

Click **Create**. Download the JSON or copy the **Client ID** and **Client secret** from the dialog.

> **Heads up:** Google still issues a "secret" for Desktop-app credentials. It's not actually a secret — anyone can extract it from a shipped binary — and Postik uses PKCE so the value isn't load-bearing. Treat it like the public client_id.

## 5. Wire the credentials into the build

Two ways:

### a) Local dev: `.cargo/config.toml`

Create `src-tauri/.cargo/config.toml` (gitignored — see below) with:

```toml
[env]
POSTIK_GOOGLE_CLIENT_ID = "your-client-id-here"
POSTIK_GOOGLE_CLIENT_SECRET = "your-client-secret-here"
```

Then `cargo build` / `npm run tauri dev` from the repo root. The values are baked into the binary at compile time via `option_env!()`.

> **Note:** `.cargo/config.toml` IS the gitignore target — never commit it. The repo's `.gitignore` already excludes `**/.cargo/config.toml`.

### b) CI / release builds: GitHub Actions secrets

In the repo's GitHub settings, add two repository secrets:

- `POSTIK_GOOGLE_CLIENT_ID`
- `POSTIK_GOOGLE_CLIENT_SECRET`

Then add an `env:` block to the build step in `.github/workflows/release.yml`:

```yaml
- uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    POSTIK_GOOGLE_CLIENT_ID: ${{ secrets.POSTIK_GOOGLE_CLIENT_ID }}
    POSTIK_GOOGLE_CLIENT_SECRET: ${{ secrets.POSTIK_GOOGLE_CLIENT_SECRET }}
  with: …
```

Until those secrets are set, the Calendar tab will show "Setup needed" in CI builds — the app still ships and works, just without Google integration.

## 6. Connect

Open Postik → Calendar tab → **Connect Google Calendar**. The default browser opens to Google's consent screen. Grant access. The app captures the redirect on a one-shot loopback listener (random port between 1024–65535) and stores the resulting refresh token in the local SQLite DB at `app_data_dir/postik.db`.

## Troubleshooting

- **"Error 400: redirect_uri_mismatch"** — Desktop OAuth clients accept `http://127.0.0.1:<any-port>/callback` by default; if you see this, double-check that you picked **Desktop app** as the credential type, not "Web application".
- **"This app isn't verified"** — Google's consent screen warns about un-verified apps. Click **Advanced → Go to Postik (unsafe)**. Verification is only needed if you publish the app for arbitrary users; for personal use the test-users list is enough.
- **Refresh token missing** — Postik requests `access_type=offline` with `prompt=consent`, which forces Google to issue a refresh token. If you ever see "Google returned no refresh_token", revoke the app at <https://myaccount.google.com/permissions> and reconnect — Google only issues a new refresh token on first consent unless you re-prompt.

## Privacy

- The scope is `calendar.readonly` — Postik can never modify or delete events.
- Tokens live in `app_data_dir/postik.db` on your machine. They're not synced anywhere.
- Disconnecting (Calendar tab → ⏻) deletes the tokens AND every event-backed note. Regular notes are untouched.
