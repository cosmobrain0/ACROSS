use ggez::{graphics::Color, Context};

use crate::{renderer::draw_rectangle, vec2d, vector::Vector};

#[derive(Debug, Clone)]
enum UIElement<'a> {
    Button(Button<'a>),
    Menu(Menu<'a>),
}

#[derive(Debug, Clone)]
struct Button<'a> {
    parent: Menu<'a>,
    position: Vector,
    size: Vector,
}

impl<'a> Button<'a> {
    #[inline]
    pub fn x(&self) -> f32 {
        self.position.x * self.parent.scale() + self.parent.position().x
    }
    #[inline]
    pub fn y(&self) -> f32 {
        self.position.y * self.parent.scale() + self.parent.position().y
    }
    #[inline]
    pub fn width(&self) -> f32 {
        self.size.x * self.parent.scale()
    }
    #[inline]
    pub fn height(&self) -> f32 {
        self.size.y * self.parent.scale()
    }

    pub fn new(position: Vector, size: Vector, parent: Menu<'a>) -> Self {
        Self {
            position,
            size,
            parent,
        }
    }

    pub fn is_hovered(&self, mouse: Vector) -> bool {
        self.x() <= mouse.x
            && self.y() <= mouse.y
            && self.x() + self.width() >= mouse.x
            && self.x() + self.height() >= mouse.y
    }

    pub fn draw(&self, ctx: &mut Context) {
        draw_rectangle(ctx, self.position, self.size, Color::BLUE);
    }
}

#[derive(Debug, Clone)]
struct Menu<'a> {
    position: Vector,
    scale: f32,
    elements: Vec<UIElement<'a>>,
    parent: Option<&'a Menu<'a>>,
}

impl<'a> Menu<'a> {
    pub fn new(position: Vector, scale: f32, parent: Option<&'a Menu<'a>>) -> Self {
        Self {
            position,
            scale,
            elements: Vec::new(),
            parent,
        }
    }

    /// There *must* be a better way
    /// TODO: find a better way
    pub fn add_elements(&mut self, elements: Vec<UIElement<'a>>) {
        self.elements = [self.elements.clone(), elements].concat();
    }

    pub fn position(&self) -> Vector {
        match self.parent {
            Some(parent) => self.position * parent.scale + parent.position(),
            None => self.position,
        }
    }

    pub fn scale(&self) -> f32 {
        match self.parent {
            Some(parent) => self.scale * parent.scale(),
            None => self.scale,
        }
    }
}

impl Default for Menu<'_> {
    fn default() -> Self {
        Self {
            position: vec2d!(0.0, 0.0),
            scale: 1.0,
            elements: vec![],
            parent: None,
        }
    }
}
