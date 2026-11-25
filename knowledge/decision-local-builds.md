# Decision: no GitHub Actions, the check and the releases run on the owner's machines

**Decided:** 2026-11-25, the owner: GitHub Actions does not work for them at the moment · applies to checks, packaging and releases

## Decision

- The repository has no `.github/workflows/`. Nothing depends on a runner.
- The gate of every session is the local check `node scripts/check.mjs` (`conventions.md`).
- A release is built on each system by a script in `scripts/`: the Windows installer on the Windows PC, the macOS dmg on the Mac, Linux later on a Linux machine or a virtual one. Each script runs the check first.
- The packages are uploaded to a GitHub release with `gh release create` and `gh release upload` from the machine that built them. The version comes from one place (`src-tauri/tauri.conf.json`), the tag is set by the script.
- Updates in the app (`tauri-plugin-updater`) read a `latest.json` from the release. The update signing key lives in the credential store of the machine that signs, never in the repository.

## Why

- Actions does not run for the owner now (2026-11-25).
- In 1.0 a run on macOS counted ten times and on Windows twice against the 3,000 monthly minutes, and the budget was spent before the month ended (`decision-stack.md` there, 2026-11-05). Since 0.11 the releases of 1.0 were built and installed locally anyway.
- A macOS package can only be built and signed on a Mac.

## Consequences

- A step that only a runner of another system would catch (a path rule of Windows, a case sensitive file system on Linux) is caught by a test of the core where possible, and by the release script of that system otherwise.
- Should Actions work again, a workflow may run the same `node scripts/check.mjs`. This decision is then revisited, the check stays the one definition.
