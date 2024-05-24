use iced::{
    button, executor, widget::Image, window, window::icon::Icon, Alignment, Application, Button,
    Column, Command, Container, Element, Length, Row, Settings, Text,
};
use image::{io::Reader as ImageReader, GenericImageView};
use std::convert::TryInto; // TryInto for converting usize to u32
use rfd::FileDialog; // FileDialog for folder selection

extern crate dirs;

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
    let width = img.width().try_into().expect("Width out of range");
    let height = img.height().try_into().expect("Height out of range");
    let raw_data = rgba.into_raw();
    let icon = Icon::from_rgba(raw_data, width, height).expect("Failed to create icon");
    Ok(icon)
}

#[derive(Default)]
struct RustCraft {
    minecraft_dir_button: button::State,
    backup_dir_button: button::State,
    schedule_backup_button: button::State,
    minecraft_directory: Option<String>,
    backup_directory: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    MinecraftDirPressed,
    BackupDirPressed,
    ScheduleBackupPressed,
    MinecraftDirectorySelected(Option<String>),
    BackupDirectorySelected(Option<String>),
}

impl Application for RustCraft {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("RustCraft - Backup Scheduler")
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
            Message::ScheduleBackupPressed => {
                if let (Some(minecraft_dir), Some(backup_dir)) =
                    (&self.minecraft_directory, &self.backup_directory)
                {
                    println!("Scheduling backup from {} to {}", minecraft_dir, backup_dir);
                } else {
                    println!("Please select both Minecraft and Backup directories");
                }
                Command::none()
            }
            Message::MinecraftDirectorySelected(path) => {
                self.minecraft_directory = path;
                println!(
                    "Selected Minecraft directory: {:?}",
                    self.minecraft_directory
                );
                Command::none()
            }
            Message::BackupDirectorySelected(path) => {
                self.backup_directory = path;
                println!("Selected Backup directory: {:?}", self.backup_directory);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        // Define the buttons and their text displays
        let minecraft_dir_button = Button::new(
            &mut self.minecraft_dir_button,
            Text::new("Select Minecraft Directory"),
        )
        .on_press(Message::MinecraftDirPressed)
        .padding(10)
        .width(Length::Units(250));

        let minecraft_dir_text = Text::new(
            self.minecraft_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()),
        )
        .size(16);

        let backup_dir_button = Button::new(
            &mut self.backup_dir_button,
            Text::new("Select Backup Directory"),
        )
        .on_press(Message::BackupDirPressed)
        .padding(10)
        .width(Length::Units(250));

        let backup_dir_text = Text::new(
            self.backup_directory
                .as_ref()
                .unwrap_or(&"No directory selected".to_string()),
        )
        .size(16);

        let schedule_backup_button = Button::new(
            &mut self.schedule_backup_button,
            Text::new("Schedule Backup"),
        )
        .on_press(Message::ScheduleBackupPressed)
        .padding(10)
        .width(Length::Units(250));

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

        let schedule_backup_column = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center)
            .push(schedule_backup_button);

        // Main content layout
        let buttons_column = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(minecraft_dir_column)
            .push(backup_dir_column)
            .push(schedule_backup_column);

        let image_column = Column::new()
            .align_items(Alignment::Center)
            .width(Length::FillPortion(1))
            .push(Image::new("assets/crea.jpeg").width(Length::Fill));

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
