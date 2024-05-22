use iced::{
    button, widget::Image, window, window::icon::Icon, Alignment, Button, Column, Container,
    Element, Length, Row, Sandbox, Settings, Text,
};
use image::{io::Reader as ImageReader, GenericImageView};
use std::convert::TryInto;

fn main() -> iced::Result {
    let icon_path = "assets/icon.ico";
    let icon = load_icon(icon_path).expect("Failed to load icon");

    RustCraft::run(Settings {
        window: window::Settings {
            size: (640, 360),
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: Some(icon),
            min_size: Some((640, 360)),
            max_size: None,
            position: window::Position::Default,
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
}

#[derive(Debug, Clone)]
enum Message {
    MinecraftDirPressed,
    BackupDirPressed,
    ScheduleBackupPressed,
}

impl Sandbox for RustCraft {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("RustCraft - Backup Scheduler")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::MinecraftDirPressed => {
                println!("Minecraft Directory Button Pressed");
            }
            Message::BackupDirPressed => {
                println!("Backup Directory Button Pressed");
            }
            Message::ScheduleBackupPressed => {
                println!("Schedule Backup Button Pressed");
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let minecraft_dir_button = Button::new(
            &mut self.minecraft_dir_button,
            Text::new("Select Minecraft Directory"),
        )
        .on_press(Message::MinecraftDirPressed)
        .padding(10)
        .width(Length::Units(250));

        let backup_dir_button = Button::new(
            &mut self.backup_dir_button,
            Text::new("Select Backup Directory"),
        )
        .on_press(Message::BackupDirPressed)
        .padding(10)
        .width(Length::Units(250));

        let schedule_backup_button = Button::new(
            &mut self.schedule_backup_button,
            Text::new("Schedule Backup"),
        )
        .on_press(Message::ScheduleBackupPressed)
        .padding(10)
        .width(Length::Units(250));

        let buttons_column = Column::new()
            .align_items(Alignment::Center)
            .spacing(10)
            .padding(20)
            .push(minecraft_dir_button)
            .push(backup_dir_button)
            .push(schedule_backup_button);

        let content = Row::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(Image::new("assets/crea.jpeg"))
            .push(buttons_column);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
