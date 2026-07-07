# RustCraft

RustCraft is a badly-named but hopefully useful Minecraft worlds backup scheduler for Windows, built with [Rust](https://www.rust-lang.org/) & [iced](https://github.com/iced-rs/iced). It's aimed at Minecraft saves, but it will happily back up any folder you point it at.

> [!NOTE]
> Your Minecraft worlds live in exactly one folder on your disk. One corrupted save, one misbehaving mod, one accidental delete, and months of building are gone. RustCraft keeps copies so that doesn't happen to you.

## Features ✨

- **Backup scheduler**: back up your worlds automatically at an interval you choose (1 to 24 hours).
- **Manual backups**: set the slider to 0 for a one-off backup.
- **Directory selection**: pick your Minecraft folder and backup destination through the UI. The dialog defaults to `AppData\Roaming\.minecraft\saves`, where Minecraft keeps its saves.
- **Notifications**: you get a system notification when a backup finishes, and another if something goes wrong.
- **Windows support**: works on any Windows version.

<p align="center">
  <img width="760px" align="center" alt="rustcraft" src="https://github.com/user-attachments/assets/faee048d-e1f7-4eb1-ab4d-b857b1f8340d" />
</p>

## Usage 📖

| Action                      | Description                                                                                         |
|-----------------------------|-----------------------------------------------------------------------------------------------------|
| **Schedule a backup**       | Set the interval (1 to 24 hours) with the slider and hit start.                                     |
| **Run a manual backup**     | Set the slider to 0 hours and hit start.                                                            |
| **Select directories**      | Use the "Select Minecraft Directory" and "Select Backup Directory" buttons.                          |
| **Notifications**           | You'll be notified when backups succeed or fail.                                                     |

## Icon attribution 🖼️
<a href="https://www.flaticon.com/authors/alfredo-creates" title="minecraft icons">Minecraft icon by Alfredo Creates, CC 3.0 BY - Flaticon</a>

## Download 🚀

Installers are on the [Releases Page](https://github.com/FrancescoCoding/rustcraft/releases).

> [!WARNING]
> Some antivirus programs may flag the installer. This happens because RustCraft isn't code-signed: signing certificates cost more than makes sense for a small free project, and without one, Windows has no way to verify where the installer came from. The source is public in this repo if you'd rather build it yourself.

## Feedback and support 📬

Found a bug or have a request? Open a [GitHub Issue](https://github.com/FrancescoCoding/rustcraft/issues) or email me at [hello@franwbu.com](mailto:hello@franwbu.com).

## Next features 🛠️

What I work on next mostly depends on requests and free time. Currently on the list:

- **Linux support**: ![70%](https://progress-bar.xyz/70) A lot of Minecraft servers run on Linux, and most of the app was written with this in mind. Needs some finishing work.
- **Backup to Drive**: ![20%](https://progress-bar.xyz/20) Back up straight to cloud storage.
- **Backup from SFTP**: ![0%](https://progress-bar.xyz/0) Pull worlds down from an SFTP server before backing them up.
