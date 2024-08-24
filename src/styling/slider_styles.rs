use iced::border::Radius;
use iced::{theme, widget::slider, Color};

pub struct MinecraftSlider;

impl slider::StyleSheet for MinecraftSlider {
    type Style = theme::Theme;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (
                    Color::from_rgb8(0x2e, 0x8b, 0x57),
                    Color::from_rgb8(0x3a, 0x7a, 0x3a),
                ),
                width: 2.0,
                border_radius: Radius::from(2.0),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 10,
                    border_radius: Radius::from(2.0),
                },
                color: Color::from_rgb8(0x3a, 0x7a, 0x3a), // Creeper green color for the handle
                border_color: Color::from_rgb8(0x2e, 0x8b, 0x57), // Darker green for the border
                border_width: 1.0,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> slider::Appearance {
        let active = self.active(_style);
        slider::Appearance {
            handle: slider::Handle {
                color: Color::from_rgb8(0x6e, 0xc1, 0x6e), // Lighter green when hovered
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, _style: &Self::Style) -> slider::Appearance {
        let hovered = self.hovered(_style);
        slider::Appearance {
            handle: slider::Handle {
                color: Color::from_rgb8(0x5e, 0xb1, 0x5e), // Slightly darker when dragging
                ..hovered.handle
            },
            ..hovered
        }
    }
}

impl From<MinecraftSlider> for theme::Slider {
    fn from(_: MinecraftSlider) -> Self {
        theme::Slider::Custom(Box::new(MinecraftSlider))
    }
}
