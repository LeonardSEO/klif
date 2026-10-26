<img align="left" width="90" height="90" src="/src/klif.svg" alt="Klif logo">

# Klif

**A desktop sideloader for iPhone and iPad that warns you before your apps expire.**

[Download](https://leonardseo.github.io/klif/) · [Releases](https://github.com/LeonardSEO/klif/releases) · [Report a problem](https://github.com/LeonardSEO/klif/issues)

<br clear="left">

---

Apps you sideload with a free Apple ID stop opening seven days after they were signed. SideStore is supposed to refresh them in the background, but the refresh can fail without telling you: the VPN drops, iOS skips the background task, or the certificate gets revoked. Most people only find out when the app won't open.

Klif shows you exactly when every sideloaded app on your device expires, and puts reminders in the calendar you already use, so they show up on all your devices before the deadline.

> [!NOTE]
> Klif is a fork of [iloader](https://github.com/nab138/iloader) by [nab138](https://github.com/nab138). It is **not affiliated with or endorsed by** iloader or its author. Please report Klif problems [here](https://github.com/LeonardSEO/klif/issues), not to the iloader project.

## Features

**New in Klif**

- **Expiry overview.** Lists every provisioning profile on the connected device, with the expiry date and the time left. Each app is colour-coded: green, then yellow at 3 days left, orange at 1 day, red once expired.
- **Calendar reminders.** One click exports an `.ics` file with an event at each app's expiry time and alerts 2 days, 1 day and 6 hours before. Open the file in Apple Calendar, Google Calendar or Outlook, and the reminders reach your phone, laptop and watch.
- **No sign-in needed for either.** Expiry dates are read straight from the device over USB or Wi-Fi via `misagent`. Your Apple ID is not used and nothing is sent to a server.

**Inherited from iloader**

- Install SideStore (or LiveContainer + SideStore), import the certificate and place the rppairing and lockdown pairing files automatically
- Import any IPA
- Error suggestions for common problems
- Manage pairing files in apps like StikDebug, SideStore and Protokolle
- View and revoke development certificates and App IDs

## Status

Klif is early-stage software.

- The expiry overview and calendar export are covered by unit tests, and reading profiles over `misagent` has been checked against a real device.
- The overview has **not yet been tested on a device with sideloaded apps installed**. Feedback from people who use SideStore every day is very welcome.
- Builds are not yet signed by Apple or Microsoft, so your system warns you the first time you open Klif. The [install steps](#install) explain how to continue. Automatic updates are switched off until signed releases exist.

## Install

Download Klif from **[leonardseo.github.io/klif](https://leonardseo.github.io/klif/)**, which picks the right file for your system, or from the [Releases page](https://github.com/LeonardSEO/klif/releases).

| Platform | File | Needs |
| --- | --- | --- |
| macOS 11+ (Apple silicon and Intel) | `.dmg` | Nothing |
| Windows 10/11 | `.exe` installer | [iTunes](https://apple.co/ms) |
| Linux x86_64 / ARM64 | `.deb`, `.rpm` or `.AppImage` | `usbmuxd` (usually preinstalled) |

### macOS

1. Open the `.dmg` and drag Klif to Applications.
2. Open Klif. macOS says it can't check the app, because Klif isn't notarized yet.
3. Go to **System Settings → Privacy & Security** and click **Open Anyway** next to Klif.
4. If macOS says the app is damaged, run this and open Klif again:

   ```bash
   xattr -dr com.apple.quarantine /Applications/Klif.app
   ```

### Windows

1. Install [iTunes](https://apple.co/ms) if you don't have it. Klif uses it to talk to your iPhone.
2. Run the installer.
3. If you see "Windows protected your PC", click **More info**, then **Run anyway**.

### Linux

1. Make sure usbmuxd is installed, for example `sudo apt install usbmuxd` or `sudo dnf install usbmuxd`.
2. Install the `.deb` or `.rpm` with your package manager, or make the AppImage executable with `chmod +x Klif*.AppImage` and run it.
3. On NixOS, run `nix run github:LeonardSEO/klif`.

## How to use

1. Plug your iPhone or iPad into your computer and open Klif.
2. Select your device. To install apps, also sign in with your Apple ID.
3. Open **Expiry Overview** in the sidebar to see when your apps expire.
4. Click **Add reminders to calendar** and open the saved `.ics` file in your calendar app.

Export the file again after each refresh or reinstall, because a refreshed app gets a new expiry date.

## Building from source

1. Install [bun](https://bun.sh) (or [Node.js](https://nodejs.org)) and [Rust](https://www.rust-lang.org/tools/install).
2. Clone the repository and `cd` into it.
3. Install dependencies with `bun i` (or `npm i`).

| Task | Command |
| --- | --- |
| Develop with hot reload | `bun tauri dev` |
| Production build | `bun tauri build` |
| Run the Rust tests | `cd src-tauri && cargo test --lib` |

## Troubleshooting

- Use **View Logs** in the app. If it shows nothing, set the log level to **Debug**.
- More detailed logs are stored here:
  - **Windows:** `%APPDATA%\io.github.leonardseo.klif\logs`
  - **macOS:** `~/Library/Application Support/io.github.leonardseo.klif/logs`
  - **Linux:** `~/.local/share/io.github.leonardseo.klif/logs/`
- If you can't solve a problem, [open an issue](https://github.com/LeonardSEO/klif/issues) and include the full error message.

## Translating

The expiry overview is available in English and Dutch. All other text comes from iloader's existing translations.

- **Fix or update a language:** open a pull request that changes `src/locales/<lang>.json`.
- **Add a language:** add it to `src/i18next.ts`, copy `src/locales/en.json` to `src/locales/<langcode>.json` and translate the strings.

```ts
const languages = [
  ["en", "English"],
  ["es", "Español"],
  // Your language here...
] as const;
```

Translations of the new `expiry` section into other languages are especially welcome.

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](/CONTRIBUTING.md).

## Credits

Klif is built on the work of many people.

- [iloader](https://github.com/nab138/iloader) by [nab138](https://github.com/nab138): the app Klif is forked from
- [isideload](https://github.com/nab138/isideload) by nab138, for installing apps, which builds on:
  - [apple-codesign-quick](https://github.com/Dadoum/apple-codesign-quick) by [Dadoum](https://github.com/Dadoum), for code signing and entitlements
  - [Impactor](https://github.com/claration/Impactor) by [claration](https://github.com/claration), as a reference for cryptography operations
  - [Sideloader](https://github.com/Dadoum/Sideloader) by Dadoum, as a reference for Apple's private developer endpoints
- [idevice](https://github.com/jkcoxson/idevice) by [jkcoxson](https://github.com/jkcoxson), for communicating with iOS devices, including reading provisioning profiles
- [idevice_pair](https://github.com/jkcoxson/idevice_pair), as a reference for pairing file management
- iloader's UI improvements by [StephenDev0](https://github.com/StephenDev0)
- Built with [Tauri](https://tauri.app)

The translations come from iloader's contributors: By3lish, TNT-333, basketshoe, baocreata, IamArayel, kkula9999, sibwaze, notmalicik, mirdukkkkk, okinaau, ChouChiu, marcinmajsc, ern775, canpng, jazoppix, eseiker, seomin0610, Ordyan777, kakik0u, lkspodmol, marcusherelammonstyle-cmd, MCI49312, Kynonim, DD00031, Toritan123, dleiferives, ShadowWLX, fkpcomposer, 474FrediFred, bovae and b1twalker. Thank you!

## License

The source code is licensed under the [MIT License](/LICENSE).

- Copyright © 2025 nab138 (iloader)
- Copyright © 2026 LeonardSEO (Klif modifications)

The name "iloader" and iloader's logos are **not** covered by the MIT License. Klif uses its own name and logo, and mentions iloader only to credit the original project. See [LICENSE-BRANDING](/LICENSE-BRANDING) for iloader's branding terms.
