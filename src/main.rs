#![windows_subsystem = "windows"]

use iced::font::{self, Font};
use iced::widget::tooltip::{Position as TooltipPosition, Tooltip};
use iced::widget::{image::Handle as ImageHandle, Button, Column, Container, Row, Slider, Text};
use iced::Color;
use iced::{
    alignment::{Horizontal, Vertical},
    executor, theme,
    time::every,
    widget::Image,
    window::{self, Icon},
    Alignment, Application, Command, Element, Length, Settings, Size, Subscription, Theme,
};
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

mod styling {
    pub mod _general_styles;
    pub mod button_styles;
    pub mod slider_styles;
}
use styling::_general_styles::text_sizes;
use styling::button_styles;
use styling::slider_styles;

pub const MONOCRAFT: Font = Font {
    family: font::Family::Name("Monocraft"),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

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

    let window_settings = window::Settings {
        // Height matches the rendered image exactly (square image at half the
        // window width), so no empty bars appear above or below it.
        size: Size {
            width: 1087f32,
            height: 543.5f32,
        },
        resizable: false,
        decorations: true,
        transparent: false,
        icon: Some(icon),
        min_size: None,
        max_size: None,
        position: window::Position::Centered,
        ..Default::default()
    };

    let settings = Settings {
        window: window_settings,
        ..Settings::default()
    };

    if let Err(e) = RustCraft::run(settings) {
        show_system_modal_message("Error", &format!("Failed to run RustCraft: {}", e));
    }
}

// Load an embedded asset as an iced image, so the app works regardless of
// the working directory it is launched from.
fn asset_image(name: &str) -> Image<ImageHandle> {
    let data = assets::get_asset(name).unwrap_or_else(|| panic!("Asset not found: {}", name));
    Image::new(ImageHandle::from_memory(data))
}

// Shorten a filesystem path for display, keeping the last two components.
fn truncate_path(path: &str) -> String {
    let components: Vec<&str> = path.split('\\').collect();
    if components.len() > 3 {
        format!("...\\{}", components[components.len() - 2..].join("\\"))
    } else {
        path.to_string()
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
    dark_theme: bool,
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
    FontLoaded(Result<(), font::Error>),
    ToggleTheme,
}

impl RustCraft {
    fn toggle_theme(&mut self) {
        self.dark_theme = !self.dark_theme;
    }

    fn update_image_path(&mut self, message: Message) {
        self.image_path = match message {
            Message::BackupCompleted => "normal.png".to_string(),
            Message::BackupError(_) => "error.png".to_string(),
            Message::StartPressed => "active.png".to_string(),
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
        let (minecraft_directory, backup_directory, schedule_hours, dark_theme) =
            config::load_configuration();
        (
            Self {
                minecraft_directory,
                backup_directory,
                schedule_hours,
                dark_theme,
                image_path: "normal.png".to_string(),
                ..Self::default()
            },
            Command::batch(vec![font::load(
                include_bytes!("../fonts/Monocraft.ttc").as_slice(),
            )
            .map(Message::FontLoaded)]),
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

                // If hours is 0 and there is an active schedule, send a signal to stop the backup thread and deactivate the schedule.
                if hours == 0 && self.active_schedule {
                    if let Some(sender) = self.backup_thread.take() {
                        let _ = sender.send(());
                        self.active_schedule = false;
                    }
                }

                if let Err(e) = config::save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                    self.dark_theme,
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
                    self.dark_theme,
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
                    self.dark_theme,
                )
                .unwrap();
                println!("Selected Backup directory: {:?}", self.backup_directory);
                Command::none()
            }

            Message::ToggleTheme => {
                self.toggle_theme();
                // Persist the theme choice across restarts
                if let Err(e) = config::save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                    self.dark_theme,
                ) {
                    println!("Error saving configuration: {}", e);
                }
                Command::none()
            }

            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let theme_toggle_button = Button::new(
            asset_image(if self.dark_theme {
                "_sun.png"
            } else {
                "_moon.png"
            })
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(42.0)),
        )
        .padding(8)
        .on_press(Message::ToggleTheme)
        .style(button_styles::TransparentButton);

        let theme_toggle = Tooltip::new(
            theme_toggle_button,
            Text::new(if self.dark_theme {
                "Switch to light theme"
            } else {
                "Switch to dark theme"
            })
            .font(MONOCRAFT)
            .size(text_sizes::SECONDARY),
            TooltipPosition::Left,
        )
        .style(theme::Container::Box);

        let start_button_text = if self.active_schedule {
            "Stop"
        } else {
            "Start"
        };

        let mut start_button = Button::new(Text::new(start_button_text).font(MONOCRAFT))
            .padding(10)
            .style(button_styles::MinecraftButton);

        // Enable start button only if both directories are selected and the schedule is not active
        if self.minecraft_directory.is_some() && self.backup_directory.is_some()
            || self.active_schedule
        {
            start_button = start_button.on_press(Message::StartPressed);
        }

        let control_buttons = Row::new().spacing(10).push(start_button);

        let mut minecraft_dir_button = Button::new(
            Text::new("Select Minecraft Directory")
                .font(MONOCRAFT)
                .size(text_sizes::PRIMARY),
        )
        .padding(10)
        .width(Length::Fixed(370f32))
        .style(button_styles::MinecraftButton);

        if !self.active_schedule {
            minecraft_dir_button = minecraft_dir_button.on_press(Message::MinecraftDirPressed);
        }

        // Show a truncated path to keep the layout tidy; the full path is in a tooltip
        let minecraft_dir_text: Element<Message> = match &self.minecraft_directory {
            Some(path) => Tooltip::new(
                Text::new(truncate_path(path))
                    .font(MONOCRAFT)
                    .size(text_sizes::SECONDARY),
                Text::new(path.clone())
                    .font(MONOCRAFT)
                    .size(text_sizes::SECONDARY),
                TooltipPosition::Bottom,
            )
            .style(theme::Container::Box)
            .into(),
            None => Text::new("No directory selected")
                .font(MONOCRAFT)
                .size(text_sizes::SECONDARY)
                .into(),
        };

        let mut backup_dir_button = Button::new(
            Text::new("Select Backup Directory")
                .font(MONOCRAFT)
                .size(text_sizes::PRIMARY),
        )
        .padding(10)
        .width(Length::Fixed(370f32))
        .style(button_styles::MinecraftButton);

        if !self.active_schedule {
            backup_dir_button = backup_dir_button.on_press(Message::BackupDirPressed);
        }

        let backup_dir_text: Element<Message> = match &self.backup_directory {
            Some(path) => Tooltip::new(
                Text::new(truncate_path(path))
                    .font(MONOCRAFT)
                    .size(text_sizes::SECONDARY),
                Text::new(path.clone())
                    .font(MONOCRAFT)
                    .size(text_sizes::SECONDARY),
                TooltipPosition::Bottom,
            )
            .style(theme::Container::Box)
            .into(),
            None => Text::new("No directory selected")
                .font(MONOCRAFT)
                .size(text_sizes::SECONDARY)
                .into(),
        };

        let schedule_slider = Slider::new(0..=24, self.schedule_hours, Message::ScheduleChanged)
            .step(1)
            .width(Length::Fixed(200f32))
            .style(slider_styles::MinecraftSlider);

        let schedule_text = if self.schedule_hours == 0 {
            Text::new("Perform a one-time backup")
                .font(MONOCRAFT)
                .size(text_sizes::SECONDARY)
        } else {
            Text::new(format!("Schedule every {} hours", self.schedule_hours))
                .font(MONOCRAFT)
                .size(text_sizes::SECONDARY)
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
            .push(Text::new("Select Backup Frequency").font(MONOCRAFT))
            .push(schedule_slider)
            .push(schedule_text);

        let timer_display: Element<Message> = if self.active_schedule {
            if let Some(last_backup_time) = self.last_backup_time {
                let elapsed = last_backup_time.elapsed().as_secs();
                let next_backup_in = (self.schedule_hours * 3600) as u64 - elapsed;
                let hours = next_backup_in / 3600;
                let minutes = (next_backup_in % 3600) / 60;
                let seconds = next_backup_in % 60;
                Text::new(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                    .size(text_sizes::SECONDARY)
                    .font(MONOCRAFT)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .into()
            } else {
                Text::new("Timer not initialized")
                    .font(MONOCRAFT)
                    .size(20)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .into()
            }
        } else {
            Text::new("").into()
        };

        let image = asset_image(&self.image_path).width(Length::Fill);

        let image_column = Column::new()
            .align_items(Alignment::Center)
            .width(Length::FillPortion(1))
            .push(image);

        let buttons_column = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .padding(20)
            .push(minecraft_dir_column)
            .push(backup_dir_column)
            .push(schedule_slider_column)
            .push(control_buttons)
            .push(timer_display);

        // Float the theme toggle in the top-right corner of the right column,
        // so the main content keeps the full window height.
        let right_column = Column::new()
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .push(
                Container::new(theme_toggle)
                    .width(Length::Fill)
                    .align_x(Horizontal::Right)
                    .padding([8, 8, 0, 0]),
            )
            .push(
                Container::new(buttons_column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y(),
            );

        let main_row = Row::new()
            .push(
                Container::new(image_column)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .center_y(),
            )
            .push(right_column);

        Container::new(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        if self.dark_theme {
            Theme::custom(
                "Dark Theme".to_string(),
                theme::Palette {
                    // Deep neutral charcoal — lets the green accents pop
                    background: Color::from_rgb8(24, 26, 30),
                    text: Color::from_rgb8(235, 235, 235),
                    primary: Color::from_rgb8(0x3a, 0x7a, 0x3a),
                    ..theme::Palette::DARK
                },
            )
        } else {
            Theme::custom(
                "Light Theme".to_string(),
                theme::Palette {
                    // Sky blue, like a clear Minecraft day
                    background: Color::WHITE,
                    text: Color::BLACK,
                    primary: Color::from_rgb8(0x3a, 0x7a, 0x3a),
                    ..theme::Palette::LIGHT
                },
            )
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.active_schedule {
            every(Duration::from_secs(1)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }
}
