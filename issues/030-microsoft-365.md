# #30 Microsoft 365 with OAuth in the system browser

**Goal:** A mailbox of Microsoft 365, which refuses passwords, is read like Gmail.
**Requirements**: FA-34 · **Depends on**: #29

## Done when

- [ ] Authorization code with PKCE for a public client. The sign-in runs on Microsoft's page in the system browser, never in a window of the app. The redirect goes to a server on the loopback address for one request and at most five minutes, with `state` checked.
- [ ] The client ID is of an app registration of the owner, from the settings, never borrowed from another program. The refresh token lies in the keychain, and access tokens only in memory.
- [ ] Microsoft's refusals are shown with their code, so a missing consent is told from a wrong ID.
- [ ] After the login the same read-only path as #29. The rows in "Outside connections" name the hosts.
