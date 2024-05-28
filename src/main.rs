use iced::{
    button, executor, slider, widget::Image, window, window::icon::Icon, Alignment, Application,
    Button, Column, Command, Container, Element, Length, Row, Settings, Slider, Text,
};
use image::{io::Reader as ImageReader, GenericImageView};
use rfd::FileDialog; // FileDialog for folder selection
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

extern crate dirs;

const CONFIG_FILE: &str = "config.json";

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

fn save_configuration(
    minecraft_dir: &Option<String>,
    backup_dir: &Option<String>,
    backup_frequency: i32,
) -> std::io::Result<()> {
    let data = json!({
        "minecraft_directory": minecraft_dir,
        "backup_directory": backup_dir,
        "backup_frequency": backup_frequency
    });
    fs::write(CONFIG_FILE, serde_json::to_string_pretty(&data)?)
}

fn load_configuration() -> (Option<String>, Option<String>, i32) {
    let path = Path::new(CONFIG_FILE);
    let mut backup_frequency = 24; // Default backup frequency in hours
    let (minecraft_dir, backup_dir) = if path.exists() {
        let data = fs::read_to_string(path).unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        let minecraft_dir = json["minecraft_directory"].as_str().map(String::from);
        let backup_dir = json["backup_directory"].as_str().map(String::from);
        if let Some(freq) = json["backup_frequency"].as_i64() {
            backup_frequency = freq as i32;
        }
        (minecraft_dir, backup_dir)
    } else {
        (None, None)
    };
    (minecraft_dir, backup_dir, backup_frequency)
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

impl Application for RustCraft {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let (minecraft_directory, backup_directory, schedule_hours) = load_configuration();
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
                if let Err(e) = save_configuration(
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
                save_configuration(
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
                save_configuration(
                    &self.minecraft_directory,
                    &self.backup_directory,
                    self.schedule_hours,
                )
                .unwrap();
                println!("Selected Backup directory: {:?}", self.backup_directory);
                Command::none()
            }
            Message::StartPressed => {
                self.active_schedule = true;
                println!("Backup schedule activated");
                Command::none()
            }
            Message::StopPressed => {
                self.active_schedule = false;
                println!("Backup schedule stopped");
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
