use super::*;
use orco::ir;

mod expression;

/// Flowchart representation of IR
#[derive(Clone)]
pub struct Flowchart {
    /// Layers of the flowchart
    pub layers: Vec<Vec<Node>>,
    /// Font to render the flowchart with
    pub font: font_kit::font::Font,
}

/// A single IR node on the flowchart
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Node {
    /// Node label
    pub label: String,
    /// Node children
    pub children: Option<std::ops::RangeInclusive<usize>>,
}

impl Default for Flowchart {
    fn default() -> Self {
        Self::new(
            font_kit::source::SystemSource::new()
                .all_fonts()
                .ok()
                .and_then(|fonts| fonts.first().and_then(|font| font.load().ok()))
                .unwrap_or_else(|| {
                    font_kit::loaders::default::Font::from_bytes(
                        include_bytes!("../../../assets/Roobert-Regular.ttf")
                            .to_vec()
                            .into(),
                        0,
                    )
                    .expect("Font loading is broken :(")
                }),
        )
    }
}

impl Flowchart {
    /// Create an empty flowchart
    pub fn new(font: font_kit::font::Font) -> Self {
        Self {
            layers: Vec::new(),
            font,
        }
    }

    /// Render this flowchart to an ril image
    pub fn render(&self) -> DrawTarget {
        let padding = Vector::new(10.0, 10.0);

        #[derive(Clone, Debug, PartialEq)]
        struct Layout<'a> {
            position: Point,
            size: Vector,
            subtree_width: f32,
            node: &'a Node,
        }

        let mut layers = self
            .layers
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|node| {
                        let size = node.size(&self.font);
                        Layout {
                            position: Point::origin(),
                            size,
                            subtree_width: size.0,
                            node,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // * Lay stuff out
        {
            // Lay out vertically
            let mut cursor_y = 0.0;
            for layer in &mut layers {
                let max_height = layer
                    .iter()
                    .map(|node| node.size.y)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.0);
                for node in layer {
                    node.position.y = cursor_y + (max_height - node.size.y) / 2.0;
                }
                cursor_y += max_height + padding.y;
            }
        }
        {
            // Lay out horizontally
            for i in (0..layers.len()).rev() {
                let (layer, next) = layers[i..].split_first_mut().unwrap();
                for node in layer {
                    if let Some(children) = &node.node.children {
                        let subtree_width = next[0][children.clone()]
                            .iter()
                            .map(|node| node.subtree_width)
                            .sum()
                            + (children.clone().count() as f32 - 1.0) * padding.x;
                        node.subtree_width = subtree_width.max(node.size.x);
                    }
                }
            }
            for i in 0..layers.len() {
                let (layer, next) = layers[i..].split_first_mut().unwrap();

                let mut cursor_x = 0.0;
                for node in layer {
                    if node.position.x != 0.0 {
                        cursor_x = node.position.x;
                    }
                    node.position.x = cursor_x + (node.subtree_width - node.size.x) / 2.0;
                    if let Some(children) = &node.node.children {
                        next[0][*children.start()].position.x = cursor_x;
                    }
                    cursor_x += node.subtree_width + padding.x;
                }
            }
        }

        // * Actually draw
        let mut image = DrawTarget::new(
            layers
                .iter()
                .map(|layer| {
                    layer
                        .last()
                        .map_or(0, |node| (node.position.x + node.size.x) as _)
                })
                .max()
                .unwrap_or(0),
            layers.last().map_or(0, |layer| {
                layer
                    .iter()
                    .map(|node| (node.position.y + node.size.y) as _)
                    .max()
                    .unwrap_or(0)
            }),
        );

        for layer in layers {
            for node in layer {
                node.node
                    .render(node.position, node.size, &self.font, &mut image);
            }
        }
        image
    }
}

impl Node {
    const PADDING: u32 = 10;

    /// Render label of this node
    pub fn label<'a>(&'a self, font: &'a Font, x: u32, y: u32) -> TextLayout<'a, Rgba> {
        TextLayout::new()
            .with_position(x, y)
            .with_basic_text(font, &self.label, Rgba::black())
            .with_align(TextAlign::Center)
            .with_vertical_anchor(VerticalAnchor::Center)
            .with_horizontal_anchor(HorizontalAnchor::Center)
    }

    /// Pre-render this node and get a size of it
    pub fn size(&self, font: &Font) -> (u32, u32) {
        let size = self.label(font, 0, 0).dimensions();
        (size.0 + Self::PADDING * 2, size.1 + Self::PADDING * 2)
        ndsle
    }

    /// Render this node
    pub fn render(
        &self,
        position: (u32, u32),
        size: (u32, u32),
        font: &Font,
        image: &mut Image<Rgba>,
    ) {
        image.draw(
            &Rectangle::at(position.0, position.1)
                .with_size(size.0, size.1)
                .with_fill(Rgba::white())
                .with_border(Border::new(Rgba::black(), 3).with_position(BorderPosition::Inset)),
        );
        image.draw(&self.label(font, position.0 + size.0 / 2, position.1 + size.1 / 2))
    }
}
