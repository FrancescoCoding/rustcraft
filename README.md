# <img src="https://wiki.bedrock.dev/assets/images/concepts/emojis/items/crafting_table.png" /> RustCraft

RustCraft is a badly-named but hopefully useful Minecraft worlds backup scheduler for Windows built with [Rust](https://www.rust-lang.org/) & [iced](https://github.com/iced-rs/iced). While it's primarily designed for backing up Minecraft worlds, it can be used to backup any files or directories, making it a versatile tool for your backup needs.

> [!NOTE]  
> In the unpredictable world of Minecraft, it's crucial to safeguard your creations. Imagine spending countless hours building intricate structures, or crafting your unique home, only to lose everything to a sudden in-game disaster, like a fire spreading uncontrollably, a Creeper explosion, or an unexpected update that corrupts your save files (mods users iykyk 💀). Your Minecraft saves are continuously evolving, reflecting your creativity and hard work. RustCraft ensures that these precious files are always backed up, giving you peace of mind and allowing you to focus on what you do best: building and exploring.

## Features ✨

- **Automatic Backup Scheduler**: Schedule backups for your Minecraft worlds at regular intervals (1 to 24 hours).
- **Manual Backup Option**: Perform a one-time backup of your Minecraft worlds (by setting the frequency scrollbar to 0).
- **Directory Selection**: Easily select the Minecraft directory and the backup destination directory using a graphical interface. The `AppData\Roaming\.minecraft\saves` folder, which contains Minecraft saves, is set as the default when you open the dialog.
- **Notifications**: Receive system notifications upon successful backups or errors.
- **Windows Compatibility**: Works on all Windows operating systems.

<p align="center">
  <img src="https://github.com/FrancescoCoding/rustcraft/assets/64712227/789ca7e4-a842-4630-9e22-84b6d12fbbc9" alt="drawing" width="760px" align="center" />
</p>

## Usage 📖

| Action                           | Description                                                                                           |
|----------------------------------|-------------------------------------------------------------------------------------------------------|
| **Schedule a Backup**            | Set the backup interval (1 to 24 hours) using the slider and click the start button.                  |
| **Perform a Manual Backup**      | Set the slider to 0 hours and click the start button to perform a one-time backup.                    |
| **Select Directories**           | Click the "Select Minecraft Directory" and "Select Backup Directory" buttons to choose directories.   |
| **Receive Notifications**        | Get system notifications for successful backups and errors.                                           |

## Icon Attribution 🖼️
<a href="https://www.flaticon.com/authors/alfredo-creates" title="minecraft icons">Minecraft icon was created by Alfredo Creates under the License CC 3.0 BY - Flaticon</a>

## Download 🚀

You can download the installers from the [Releases Page](https://github.com/FrancescoCoding/rustcraft/releases).

> [!WARNING]
> Due to the nature of unsigned software, some antivirus programs may flag this installer as potentially harmful. This is a common occurrence with software that is not code-signed with a certificate from a trusted Certificate Authority (CA).
> 
> - **Why This?**  
> RustCraft is a small project, and I currently do not have the resources to pay for a (very costly) code-signing certificate. Unsigned software can trigger antivirus warnings because it lacks a digital signature that verifies its origin and integrity.

## Feedback and Support 📬

If you encounter any issues or have concerns, please reach out to me through [GitHub Issues](https://github.com/FrancescoCoding/rustcraft/issues) or my support email [hello@franwbu.com](mailto:hello@franwbu.com).

## Next Features 🛠️

I am continuously working on RustCraft, depending on features requests or time. The next features I will focus on include:

- **Linux Compatibility**: ![70%](https://progress-bar.xyz/70) A lot of Minecraft servers run on Linux, and most of the app is pre-built with this in mind. Still needs some work.
- **Backup to Drive**: ![20%](https://progress-bar.xyz/20) Allow users to backup their Minecraft worlds directly to cloud storage services.
- **Backup from SFTP**: ![0%](https://progress-bar.xyz/0) Enable users to backup their Minecraft worlds from an SFTP server.

Thank you for your understanding and support!
