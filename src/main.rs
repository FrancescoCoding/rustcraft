use chrono::prelude::*;
use iced::{
    button, executor, slider, widget::Image, window, window::icon::Icon, Alignment, Application,
    Button, Column, Command, Container, Element, Length, Row, Settings, Slider, Text,
};
use image::{io::Reader as ImageReader, GenericImageView};
use rfd::FileDialog; // FileDialog for folder selection
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{
    fs, io,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
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
        // Log the error and notify the user
        eprintln!("Failed to copy directory: {:?}", result);
        show_system_modal_message(
            "Backup Error",
            "An error occurred during the backup process. Please check the logs for more details.",
        );
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
    stop_button: button::State,
    active_schedule: bool,
    background_thread: Option<JoinHandle<()>>,
    should_continue: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
enum Message {
    MinecraftDirPressed,
    BackupDirPressed,
    ScheduleChanged(i32),
    MinecraftDirectorySelected(Option<String>),
    BackupDirectorySelected(Option<String>),
    StartPressed,
    StopPressed,
}

// Functions for managing background tasks
impl RustCraft {
    fn start_backup_task(&mut self) {
        let src_dir = self.minecraft_directory.clone().unwrap();
        let dst_dir = self.backup_directory.clone().unwrap();
        let frequency = self.schedule_hours;

        let should_continue = self.should_continue.clone();

        self.background_thread = Some(thread::spawn(move || {
            // Check if frequency is zero for a single immediate backup
            if frequency == 0 {
                println!("Performing a single backup...");
                if let Err(e) = copy_directory(Path::new(&src_dir), Path::new(&dst_dir)) {
                    eprintln!("Error during file copying: {}", e);
                }
                println!("Single backup completed.");
            } else {
                // Regular scheduled backups
                while should_continue.load(Ordering::SeqCst) {
                    println!("Scheduled backup initiated...");
                    if let Err(e) = copy_directory(Path::new(&src_dir), Path::new(&dst_dir)) {
                        eprintln!("Error during file copying: {}", e);
                        break; // Exit the loop on error
                    }
                    println!("Backup successful, next backup in {} hours", frequency);
                    thread::sleep(Duration::from_secs(frequency as u64 * 3600));
                }
            }
        }));
    }

    fn stop_backup_task(&mut self) {
        self.should_continue.store(false, Ordering::SeqCst);
        if let Some(handle) = self.background_thread.take() {
            if let Err(e) = handle.join() {
                eprintln!("Failed to join the thread: {:?}", e);
            }
        }
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
                let default_path = "C:\\Users\\User\\AppData\\Roaming\\.minecraft\\saves";
                let path = FileDialog::new().set_directory(default_path).pick_folder();

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
                let default_path = dirs::desktop_dir().expect("Failed to find desktop directory");
                let path = FileDialog::new().set_directory(default_path).pick_folder();

                Command::perform(
                    async move {
                        Message::BackupDirectorySelected(
                            path.map(|p| p.to_string_lossy().into_owned()),
                        )
                    },
                    |p| p,
                )
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
            Message::StartPressed => {
                if self.minecraft_directory.is_some()
                    && self.backup_directory.is_some()
                    && !self.active_schedule
                {
                    self.start_backup_task();
                    self.active_schedule = true;
                }
                Command::none()
            }
            Message::StopPressed => {
                self.stop_backup_task();
                self.active_schedule = false;
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

        let mut stop_button = Button::new(&mut self.stop_button, Text::new("Stop")).padding(10);

        if self.active_schedule {
            stop_button = stop_button.on_press(Message::StopPressed);
        }

        let control_buttons = Row::new().spacing(10).push(start_button).push(stop_button);

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

        let schedule_text =
            Text::new(format!("Schedule every {} hours", self.schedule_hours)).size(16);

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

        let image_path = if self.active_schedule {
            "assets/active.png"
        } else {
            "assets/normal.jpeg"
        };

        let image = Image::new(image_path).width(Length::Fill);

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
