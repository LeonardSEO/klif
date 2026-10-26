# Contributing to Klif

Thanks for helping out! Klif is a small project, so a few guidelines keep things manageable.

## Where does the change belong?

Klif is a fork of [iloader](https://github.com/nab138/iloader). The core sideloading work (Apple ID sign-in, code signing, installing) lives in [isideload](https://github.com/nab138/isideload), and device communication lives in [idevice](https://github.com/jkcoxson/idevice).

- If a bug is in one of those libraries, fix it there so everyone benefits.
- Report problems that only happen in Klif here.
- Don't report Klif problems to the iloader project.

## Issues

- Search existing issues first.
- Fill out the issue template, including the full error message and your Klif, iOS and OS versions.

## Pull requests

- Keep each pull request focused on one change.
- Run the checks before you open it:
  - `bun run build`
  - `cd src-tauri && cargo test --lib`
  - `cargo clippy --lib`
- New user-facing text goes into `src/locales/en.json`. Translations into other languages are welcome.
- Using AI tools is fine, but you are responsible for the code. Understand and test everything you submit, and mention in the description that you used AI.

## No piracy

Klif is for installing apps you are allowed to use. Issues and pull requests that support piracy will be closed.
