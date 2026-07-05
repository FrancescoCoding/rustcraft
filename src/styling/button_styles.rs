use iced::border::Radius;
use iced::{
    theme,
    widget::{button, container},
    Background, Border, Color, Shadow, Vector,
};

pub struct MinecraftButton;

impl button::StyleSheet for MinecraftButton {
    type Style = theme::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb8(0x3a, 0x7a, 0x3a))), // Creeper green color
            border: Border {
                color: Color::from_rgb8(0x2e, 0x8b, 0x57), // Darker green for the border
                width: 1.0,
                radius: Radius::from(3.0),
            },
            shadow: Shadow {
                color: Color::from_rgb8(0x00, 0x00, 0x00),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 1.0,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb8(0x4a, 0x8b, 0x4a))), // Lighter green when hovered
            ..self.active(_style)
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow: Default::default(),
            ..self.hovered(_style)
        }
    }
}

impl From<MinecraftButton> for theme::Button {
    fn from(_: MinecraftButton) -> Self {
        theme::Button::Custom(Box::new(MinecraftButton))
    }
}

pub struct TransparentButton;

impl button::StyleSheet for TransparentButton {
    type Style = theme::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            shadow: Shadow::default(),
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
}

impl From<TransparentButton> for theme::Button {
    fn from(_: TransparentButton) -> Self {
        theme::Button::Custom(Box::new(TransparentButton))
    }
}

pub struct OverlayContainer;

impl container::StyleSheet for OverlayContainer {
    type Style = theme::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: None,
            border: Border::default(),
            shadow: Shadow::default(),
            text_color: None,
        }
    }
}

impl From<OverlayContainer> for theme::Container {
    fn from(_: OverlayContainer) -> Self {
        theme::Container::Custom(Box::new(OverlayContainer))
    }
}
