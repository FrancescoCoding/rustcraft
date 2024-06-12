use chrono::prelude::*;
use iced::{
    button, executor, slider, widget::Image, window, window::icon::Icon, Alignment, Application,
    Button, Column, Command, Container, Element, Length, Row, Settings, Slider, Text,
};
use image::{io::Reader as ImageReader, GenericImageView};
use rfd::FileDialog; // FileDialog for folder selection
use std::{
    fs, io,
    path::{Path, PathBuf},
};

extern crate dirs;

mod config;
extern crate winapi;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SYSTEMMODAL};

#[cfg(target_os = "windows")]
fn show_system_modal_message(title: &str, message: &str) {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONINFORMATION | MB_SYSTEMMODAL,
        );
    }
}

// Log the message to stderr on non-Windows platforms
#[cfg(not(target_os = "windows"))]
fn show_system_modal_message(title: &str, message: &str) {
    eprintln!("{}: {}", title, message);
}

fn main() -> iced::Result {
    let icon_path = "assets/icon.ico";
    let icon = load_icon(icon_path).expect("Failed to load icon");

    RustCraft::run(Settings {
        window: window::Settings {
            size: (1087, 533),
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: Some(icon),
            min_size: Some((640, 360)),
            max_size: None,
            position: window::Position::Centered,
        },
        ..Settings::default()
    })
}

fn load_icon(path: &str) -> Result<Icon, image::ImageError> {
    let img = ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();
    let width = img.width();
    let height = img.height();
    let raw_data = rgba.into_raw();
    Ok(Icon::from_rgba(raw_data, width, height).unwrap())
}

fn copy_directory(src: &Path, dst: &Path) -> io::Result<()> {
    println!("Attempting to copy from {:?} to {:?}", src, dst);
    let local: DateTime<Local> = Local::now();
    let timestamp = local.format("%d.%m.%Y %H.%M").to_string(); // Ensure no illegal characters for file paths
    let dst_with_timestamp = dst.join(timestamp);
    println!("Creating directory: {:?}", dst_with_timestamp);

    fs::create_dir_all(&dst_with_timestamp)?;

    // Recursively copy all contents from src to the new destination directory
    let result = copy_contents_recursively(src, src, &dst_with_timestamp);

    if result.is_ok() {
        show_system_modal_message(
            "Backup Notification",
            "Backup done. Your Minecraft worlds have been successfully saved.",
        );
    } else {
        let err_msg = format!("Failed to copy directory: {:?}", result.unwrap_err());
        show_system_modal_message("Backup Error", &err_msg);
        // Return the error with details
        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
    }

    result
}

/// Recursively copies contents from the source directory to the destination directory, maintaining the structure.
fn copy_contents_recursively(base: &Path, src: &Path, dst: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        // Get the relative path with respect to the base
        let relative_path = path.strip_prefix(base).unwrap();
        let destination_path = dst.join(relative_path);

        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&destination_path)?;
            // Recursive call to handle subdirectories
            copy_contents_recursively(base, &path, dst)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?; // Ensure the directory exists
            }
            println!("Copying file {:?} to {:?}", path, destination_path);
            fs::copy(&path, &destination_path)?;
        }
    }
    Ok(())
}

#[derive(Default)]
struct RustCraft {
    minecraft_dir_button: button::State,
    backup_dir_button: button::State,
    schedule_slider: slider::State,
    schedule_hours: i32,
    minecraft_directory: Option<String>,
    backup_directory: Option<String>,
    start_button: button::State,
    active_schedule: bool,
    image_path: String,
}

#[derive(Debug, Clone)]
enum Message {
    MinecraftDirPressed,
    BackupDirPressed,
    ScheduleChanged(i32),
    MinecraftDirectorySelected(Option<String>),
    BackupDirectorySelected(Option<String>),
    StartPressed,
    BackupCompleted,
    BackupError(String),
}

impl RustCraft {
    fn update_image_path(&mut self, message: Message) {
        self.image_path = match message {
            Message::BackupCompleted => "assets/normal.jpeg".to_string(),
            Message::BackupError(_) => "assets/error.png".to_string(),
            Message::StartPressed => "assets/active.png".to_string(),
            _ => self.image_path.clone(),
        };
    }

    fn get_minecraft_default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|path| path.join("AppData\\Roaming\\.minecraft\\saves"))
    }
}

impl Application for RustCraft {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let (minecraft_directory, backup_directory, schedule_hours) = config::load_configuration();
        (
            Self {
                minecraft_directory,
                backup_directory,
                schedule_hours,
                image_path: "assets/normal.jpeg".to_string(),
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("RustCraft - Worlds Backup Scheduler")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::MinecraftDirPressed => {
                let initial_directory = self
                    .minecraft_directory
                    .clone()
                    .map(PathBuf::from)
                    .or_else(RustCraft::get_minecraft_default_path);
                let path = FileDialog::new()
                    .set_directory(initial_directory.unwrap_or_else(|| PathBuf::from(".")))
                    .pick_folder();

                Command::perform(
                    async move {
                        Message::MinecraftDirectorySelected(
                            path.map(|p| p.to_string_lossy().into_owned()),
                        )
                    },
                    |p| p,
                )
            }
            Message::ScheduleChanged(hours) => {
                self.schedule_hours = hours;
                if let Err(e) = config::save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                ) {
                    println!("Error saving configuration: {}", e);
                }
                Command::none()
            }
            Message::BackupDirPressed => {
                // Check if a backup directory is already specified, otherwise default to the desktop directory
                let initial_directory = self
                    .backup_directory
                    .clone()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(".")));

                let path = FileDialog::new()
                    .set_directory(initial_directory)
                    .pick_folder();

                Command::perform(
                    async move {
                        Message::BackupDirectorySelected(
                            path.map(|p| p.to_string_lossy().into_owned()),
                        )
                    },
                    |p| p,
                )
            }

            Message::StartPressed => {
                self.active_schedule = true;
                self.update_image_path(Message::StartPressed);

                let src_dir = self.minecraft_directory.clone().unwrap();
                let dst_dir = self.backup_directory.clone().unwrap();
                Command::perform(
                    async move {
                        match copy_directory(Path::new(&src_dir), Path::new(&dst_dir)) {
                            Ok(_) => Message::BackupCompleted,
                            Err(e) => Message::BackupError(e.to_string()),
                        }
                    },
                    |res| res,
                )
            }

            Message::BackupCompleted => {
                self.active_schedule = false;
                self.update_image_path(Message::BackupCompleted);
                Command::none()
            }
            Message::BackupError(err_msg) => {
                self.active_schedule = false;
                self.update_image_path(Message::BackupError("assets/error.png".to_string()));
                show_system_modal_message("Backup Error", &err_msg);
                self.image_path = "assets/error.png".to_string();
                Command::none()
            }

            Message::MinecraftDirectorySelected(path) => {
                self.minecraft_directory = path;
                config::save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                )
                .unwrap();
                println!(
                    "Selected Minecraft directory: {:?}",
                    self.minecraft_directory
                );
                Command::none()
            }
            Message::BackupDirectorySelected(path) => {
                self.backup_directory = path;
                config::save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                )
                .unwrap();
                println!("Selected Backup directory: {:?}", self.backup_directory);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut start_button = Button::new(&mut self.start_button, Text::new("Start")).padding(10);

        // Enable start button only if both directories are selected and the schedule is not active
        if !self.active_schedule
            && self.minecraft_directory.is_some()
            && self.backup_directory.is_some()
        {
            start_button = start_button.on_press(Message::StartPressed);
        }

        let control_buttons = Row::new().spacing(10).push(start_button);

        let mut minecraft_dir_button = Button::new(
            &mut self.minecraft_dir_button,
            Text::new("Select Minecraft Directory"),
        )
        .padding(10)
        .width(Length::Units(250));

        if !self.active_schedule {
            minecraft_dir_button = minecraft_dir_button.on_press(Message::MinecraftDirPressed);
        }

        let minecraft_dir_text = Text::new(
            self.minecraft_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()),
        )
        .size(16);

        let mut backup_dir_button = Button::new(
            &mut self.backup_dir_button,
            Text::new("Select Backup Directory"),
        )
        .padding(10)
        .width(Length::Units(250));

        if !self.active_schedule {
            backup_dir_button = backup_dir_button.on_press(Message::BackupDirPressed);
        }

        let backup_dir_text = Text::new(
            self.backup_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()),
        )
        .size(16);

        let schedule_slider = Slider::new(
            &mut self.schedule_slider,
            0..=24,
            self.schedule_hours,
            Message::ScheduleChanged,
        )
        .step(1)
        .width(Length::Units(200));

        let schedule_text = if self.schedule_hours == 0 {
            Text::new("Perform a one-time backup").size(16)
        } else {
            Text::new(format!("Schedule every {} hours", self.schedule_hours)).size(16)
        };

        let minecraft_dir_column = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(minecraft_dir_button)
            .push(minecraft_dir_text);

        let backup_dir_column = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center)
            .push(backup_dir_button)
            .push(backup_dir_text);

        let schedule_slider_column = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center)
            .push(Text::new("Select Backup Frequency"))
            .push(schedule_slider)
            .push(schedule_text);

        let image = Image::new(self.image_path.clone()).width(Length::Fill);

        let image_column = Column::new()
            .align_items(Alignment::Center)
            .width(Length::FillPortion(1))
            .push(image);

        let buttons_column = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(minecraft_dir_column)
            .push(backup_dir_column)
            .push(schedule_slider_column)
            .push(control_buttons);

        let content = Row::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(image_column)
            .push(buttons_column.width(Length::FillPortion(1)));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
