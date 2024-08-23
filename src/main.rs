#![windows_subsystem = "windows"]

use iced::{alignment::{Horizontal, Vertical}, executor, time::every, widget::Image, window::{self, Icon}, Alignment, Application, Command, Element, Length, Settings, Subscription, Size, Theme};
use iced::widget::{Button, Column, Container, Row, Slider, Text};
use rfd::FileDialog; // FileDialog for folder selection (cross-platform)

use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

mod assets;
mod config;
mod file_operations;
mod notification;
extern crate dirs;
extern crate winapi;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

#[cfg(target_os = "windows")]
fn show_system_modal_message(title: &str, message: &str) {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

// Log the message to stderr on non-Windows platforms
#[cfg(not(target_os = "windows"))]
fn show_system_modal_message(title: &str, message: &str) {
    eprintln!("{}: {}", title, message);
}

fn main() {
    let icon = load_icon().expect("Failed to load icon");

    let settings: Settings<()> = Settings {
        window: window::Settings {
            size: Size { width: 1087f32, height: 533f32 },
            resizable: true,
            decorations: true,
            transparent: false,
            icon: Some(icon),
            min_size: Some(Size { width: 640f32, height: 340f32}),
            max_size: None,
            position: window::Position::Centered,
            ..Default::default()
        },
        ..Settings::default()
    };

    if let Err(e) = RustCraft::run(settings) {
        show_system_modal_message("Error", &format!("Failed to run RustCraft: {}", e));
    }
}

fn load_icon() -> Result<Icon, image::ImageError> {
    let icon_data = assets::get_asset("icon.ico").expect("Icon not found in assets");
    let img = image::load_from_memory(&icon_data)?.to_rgba8();
    let width = img.width();
    let height = img.height();
    let raw_data = img.into_raw();
    Ok(window::icon::from_rgba(raw_data, width, height).unwrap())
}

#[derive(Default)]
struct RustCraft {
    schedule_hours: i32,
    minecraft_directory: Option<String>,
    backup_directory: Option<String>,
    active_schedule: bool,
    image_path: String,
    backup_thread: Option<Sender<()>>,
    timer_text: String,
    last_backup_time: Option<Instant>,
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
    Tick(Instant),
}

impl RustCraft {
    fn update_image_path(&mut self, message: Message) {
        self.image_path = match message {
            Message::BackupCompleted => "assets/normal.png".to_string(),
            Message::BackupError(_) => "assets/error.png".to_string(),
            Message::StartPressed => "assets/active.png".to_string(),
            _ => self.image_path.clone(),
        };
    }
    fn get_minecraft_default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|path| path.join("AppData\\Roaming\\.minecraft\\saves"))
    }

    fn start_backup_thread(&mut self, hours: i32) {
        self.last_backup_time = Some(Instant::now());

        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();
        let src_dir = self.minecraft_directory.clone().unwrap();
        let dst_dir = self.backup_directory.clone().unwrap();
        thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            if let Err(e) =
                file_operations::copy_directory(Path::new(&src_dir), Path::new(&dst_dir))
            {
                show_system_modal_message("Backup Error", &e.to_string());
            } else {
                notification::trigger_notification(true, None);
            }
            thread::sleep(Duration::from_secs((hours * 3600) as u64)); // Convert hours to seconds
        });
        self.backup_thread = Some(tx);
    }
}

impl Application for RustCraft {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let (minecraft_directory, backup_directory, schedule_hours) = config::load_configuration();
        (
            Self {
                minecraft_directory,
                backup_directory,
                schedule_hours,
                image_path: "assets/normal.png".to_string(),
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
            Message::Tick(now) => {
                if let Some(last_backup_time) = self.last_backup_time {
                    let elapsed = now.duration_since(last_backup_time);
                    let seconds_since_last_backup = elapsed.as_secs();
                    let total_seconds_for_backup = (self.schedule_hours * 3600) as u64;

                    if seconds_since_last_backup >= total_seconds_for_backup {
                        // Reset the timer if the scheduled time has elapsed
                        self.last_backup_time = Some(now);
                        self.timer_text = format!("{:02}:{:02}:{:02}", self.schedule_hours, 0, 0);
                    } else {
                        let seconds_remaining =
                            total_seconds_for_backup - seconds_since_last_backup;
                        let hours = seconds_remaining / 3600;
                        let minutes = (seconds_remaining % 3600) / 60;
                        let seconds = seconds_remaining % 60;
                        self.timer_text = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
                    }
                } else {
                    // Initialize the timer if it hasn't been set
                    self.last_backup_time = Some(now);
                    self.timer_text = format!("{:02}:{:02}:{:02}", self.schedule_hours, 0, 0);
                }
                Command::none()
            }

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
                if self.active_schedule {
                    if let Some(sender) = self.backup_thread.take() {
                        let _ = sender.send(()); // Signal to stop the thread
                    }
                    self.active_schedule = false;
                    self.update_image_path(Message::BackupCompleted);
                } else if self.schedule_hours == 0 {
                    // Perform an immediate backup without threading
                    let src_dir = self.minecraft_directory.clone().unwrap();
                    let dst_dir = self.backup_directory.clone().unwrap();
                    match file_operations::copy_directory(Path::new(&src_dir), Path::new(&dst_dir))
                    {
                        Ok(_) => {
                            self.update_image_path(Message::BackupCompleted);
                            notification::trigger_notification(true, None);
                        }
                        Err(e) => {
                            let error_message = format!("Backup failed: {}", e);
                            self.update_image_path(Message::BackupError(error_message.clone()));
                            notification::trigger_notification(false, Some(&error_message));
                        }
                    }
                } else {
                    self.start_backup_thread(self.schedule_hours);
                    self.active_schedule = true;
                    self.update_image_path(Message::StartPressed);
                }
                Command::none()
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

    fn view(&self) -> Element<Self::Message> {
        let start_button_text = if self.active_schedule {
            "Stop"
        } else {
            "Start"
        };
        let mut start_button =
            Button::new(Text::new(start_button_text)).padding(10);

        // Enable start button only if both directories are selected and the schedule is not active
        if self.minecraft_directory.is_some() && self.backup_directory.is_some()
            || self.active_schedule
        {
            start_button = start_button.on_press(Message::StartPressed);
        }

        let control_buttons = Row::new().spacing(10).push(start_button);

        let mut minecraft_dir_button = Button::new(
            Text::new("Select Minecraft Directory"),
        )
        .padding(10)
        .width(Length::Fixed(250f32));

        if !self.active_schedule {
            minecraft_dir_button = minecraft_dir_button.on_press(Message::MinecraftDirPressed);
        }

        let minecraft_dir_text = Text::new(
            self.minecraft_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()).clone()
        )
        .size(16);

        let mut backup_dir_button = Button::new(
            Text::new("Select Backup Directory"),
        )
        .padding(10)
        .width(Length::Fixed(250f32));

        if !self.active_schedule {
            backup_dir_button = backup_dir_button.on_press(Message::BackupDirPressed);
        }

        let backup_dir_text = Text::new(
            self.backup_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()).clone()
        )
        .size(16);

        let schedule_slider = Slider::new(
            0..=24,
            self.schedule_hours,
            Message::ScheduleChanged,
        )
        .step(1)
        .width(Length::Fixed(200f32));

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

        let timer_display: Element<Message> = if self.active_schedule {
            if let Some(last_backup_time) = self.last_backup_time {
                let elapsed = last_backup_time.elapsed().as_secs();
                let next_backup_in = (self.schedule_hours * 3600) as u64 - elapsed;
                let hours = next_backup_in / 3600;
                let minutes = (next_backup_in % 3600) / 60;
                let seconds = next_backup_in % 60;
                Text::new(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                    .size(20)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .into()
            } else {
                Text::new("Timer not initialized")
                    .size(20)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .into()
            }
        } else {
            Text::new("").into()
        };

        let buttons_column = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(minecraft_dir_column)
            .push(backup_dir_column)
            .push(schedule_slider_column)
            .push(control_buttons)
            .push(timer_display);

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

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.active_schedule {
            every(Duration::from_secs(1)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }
}
