use std::{cell::RefCell, rc::Rc};

use ggez::{
    graphics::{self, Color},
    Context,
};

use crate::{
    renderer::{draw_rectangle, draw_text},
    vec2d,
    vector::Vector,
    MainState,
};

pub enum UIElement<'a, T> {
    Button(Button<'a, T>),
    Menu(Menu<'a, T>),
    DragButton(DragButton<'a, T>),
}

impl<T> UIElement<'_, T> {
    pub fn position(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.x(), x.y()),
            UIElement::Menu(x) => x.position(),
            UIElement::DragButton(x) => vec2d![x.button().x(), x.button().y()],
        }
    }

    pub fn size(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.width(), x.height()),
            UIElement::Menu(x) => x.size(),
            UIElement::DragButton(x) => vec2d![x.button().width(), x.button().height()],
        }
    }

    pub fn local_size(&self) -> Vector {
        match self {
            UIElement::Button(x) => x.size,
            UIElement::Menu(x) => x.local_size(),
            UIElement::DragButton(x) => x.button.size,
        }
    }

    pub fn local_position(&self) -> Vector {
        match self {
            UIElement::Button(x) => x.position,
            UIElement::Menu(x) => x.position,
            UIElement::DragButton(x) => x.button.position,
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        match self {
            UIElement::Button(x) => x.draw(ctx),
            UIElement::Menu(x) => x.draw(ctx),
            UIElement::DragButton(x) => x.button().draw(ctx),
        }
    }

    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        match self {
            UIElement::Menu(x) => x.input_start(position, state),
            UIElement::DragButton(x) => x.input_start(position, state),
            _ => (),
        }
    }

    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        match self {
            UIElement::Menu(x) => x.input_moved(position, movement, state),
            UIElement::DragButton(x) => x.input_moved(position, movement, state),
            _ => (),
        }
    }

    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        match self {
            UIElement::Button(x) => x.input_released(position, state),
            UIElement::Menu(x) => x.input_released(position, state),
            UIElement::DragButton(x) => x.input_released(position, state),
        }
    }
}

/// TODO: why is parent not just a &'a Menu?
pub struct Button<'a, T> {
    parent: Rc<RefCell<Menu<'a, T>>>,
    position: Vector,
    size: Vector,
    callback: fn(&mut T),
    text: String,
}

impl<'a, T> Button<'a, T> {
    pub fn x(&self) -> f32 {
        self.position.x * self.parent.borrow().scale() + self.parent.borrow().position().x
    }
    pub fn y(&self) -> f32 {
        self.position.y * self.parent.borrow().scale() + self.parent.borrow().position().y
    }
    pub fn width(&self) -> f32 {
        self.size.x * self.parent.borrow().scale()
    }
    pub fn height(&self) -> f32 {
        self.size.y * self.parent.borrow().scale()
    }

    pub fn new(
        position: Vector,
        size: Vector,
        parent: Rc<RefCell<Menu<'a, T>>>,
        callback: fn(&mut T),
        text: &str,
    ) -> Self
    where
        T: Sized,
    {
        Self {
            position,
            size,
            parent,
            callback,
            text: text.to_owned(),
        }
    }

    pub fn local_hovers(&self, mouse: Vector) -> bool {
        self.position.x <= mouse.x
            && self.position.y <= mouse.y
            && self.position.x + self.size.x >= mouse.x
            && self.position.y + self.size.y >= mouse.y
    }

    pub fn click(&self, state: &mut T) {
        let callback = self.callback;
        callback(state);
    }

    pub fn draw(&self, ctx: &mut Context) {
        draw_rectangle(
            ctx,
            vec2d!(self.x(), self.y()),
            vec2d!(self.width(), self.height()),
            Color::BLUE,
        );

        draw_text(
            ctx,
            &self.text,
            vec2d!(self.x(), self.y() + (self.height() - 32.0) / 2.0),
            None,
            Some((self.size, graphics::Align::Center)),
            Color::BLACK,
        );
    }

    pub fn input_released(&self, position: Vector, state: &mut T) {
        if self.local_hovers(position) {
            self.click(state);
        }
    }
}

impl<'a, T> Into<UIElement<'a, T>> for Button<'a, T> {
    fn into(self) -> UIElement<'a, T> {
        UIElement::Button(self)
    }
}

pub struct Menu<'a, T> {
    position: Vector,
    scale: f32,
    pub elements: Vec<UIElement<'a, T>>,
    parent: Option<&'a Menu<'a, T>>,
}

impl<'a, T> Menu<'a, T> {
    pub fn new(position: Vector, scale: f32, parent: Option<&'a Menu<'a, T>>) -> Self {
        Self {
            position,
            scale,
            elements: Vec::new(),
            parent,
        }
    }

    /// There *must* be a better way
    /// TODO: find a better way
    pub fn add_elements(&mut self, elements: Vec<UIElement<'a, T>>) {
        self.elements.reserve(elements.len());
        for element in elements {
            self.elements.push(element);
        }
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

    /// Returns (top_left, bottom_right)
    pub fn bounds(&self) -> (Vector, Vector) {
        let initial: Vector = self.elements[0].position();
        let initial: (Vector, Vector) = (initial, initial);
        self.elements.iter().fold(initial, |bounds, element| {
            let position = element.position();
            let size = element.size();
            (bounds.0.min(position), bounds.1.max(position + size))
        })
    }

    pub fn local_bounds(&self) -> (Vector, Vector) {
        let initial: Vector = self.elements[0].local_position();
        let initial: (Vector, Vector) = (initial, initial);
        self.elements.iter().fold(initial, |bounds, element| {
            let position = element.local_position();
            let size = element.local_size();
            (bounds.0.min(position), bounds.1.max(position + size))
        })
    }

    pub fn size(&self) -> Vector {
        let bounds = self.bounds();
        bounds.1 - bounds.0
    }

    pub fn local_size(&self) -> Vector {
        let bounds = self.local_bounds();
        bounds.1 - bounds.0
    }

    pub fn draw(&self, ctx: &mut Context) {
        let bounds = self.bounds();
        draw_rectangle(
            ctx,
            bounds.0,
            bounds.1 - bounds.0,
            Color::new(1.0, 0.0, 0.0, 0.3),
        );
        self.elements.iter().for_each(|x| x.draw(ctx));
    }

    pub fn local_position(&self) -> Vector {
        self.local_bounds().0
    }

    pub fn hovers(&self, global_position: Vector) -> bool {
        global_position.x >= self.position().x
            && global_position.x <= self.position().x + self.size().x
            && global_position.y >= self.position().y
            && global_position.y <= self.position().y + self.size().y
    }

    pub fn local_hovers(&self, position: Vector) -> bool {
        (position.x >= self.local_position().x)
            && (position.y >= self.local_position().y)
            && (position.x <= self.local_position().x + self.local_size().x)
            && (position.y <= self.local_position().y + self.local_size().y)
    }

    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        if self.local_hovers(position) {
            let local_position = self.position;
            for element in self.elements.iter_mut() {
                element.input_released((position - local_position) / self.scale, state);
            }
        }
    }

    pub fn set_position(&mut self, position: Vector) {
        self.position = position;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        if self.local_hovers(position) {
            let local_position = self.position;
            for element in self.elements.iter_mut() {
                element.input_moved(
                    (position - local_position) / self.scale,
                    movement / self.scale,
                    state,
                )
            }
        }
    }

    /// TODO: make a hovered method to avoid repetition?
    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        if self.local_hovers(position) {
            let local_position = self.position;
            for element in self.elements.iter_mut() {
                element.input_start((position - local_position) / self.scale, state);
            }
        }
    }
}

impl<'a, T> Into<UIElement<'a, T>> for Menu<'a, T> {
    fn into(self) -> UIElement<'a, T> {
        UIElement::Menu(self)
    }
}

impl<T> Default for Menu<'_, T> {
    fn default() -> Self {
        Self {
            position: vec2d!(0.0, 0.0),
            scale: 1.0,
            elements: vec![],
            parent: None,
        }
    }
}

/// TODO: maybe use composition here?
/// Actually no, that shuold be done now, not later
pub struct DragButton<'a, T> {
    parent: Rc<RefCell<Menu<'a, T>>>,
    button: Button<'a, T>,
    drag_start: Option<Vector>,
    /// self.start_callback(position, state);
    start_callback: fn(Vector, &mut T),
    /// self.moved_callback(start, position, movement, state);
    moved_callback: fn(Vector, Vector, Vector, &mut T),
    /// self.released_callback(start, position, state);
    released_callback: fn(Vector, Vector, &mut T),
}
impl<'a, T> DragButton<'a, T> {
    pub fn new(
        position: Vector,
        size: Vector,
        parent: Rc<RefCell<Menu<'a, T>>>,
        start_callback: fn(Vector, &mut T),
        moved_callback: fn(Vector, Vector, Vector, &mut T),
        released_callback: fn(Vector, Vector, &mut T),
        text: &str,
    ) -> Self
    where
        T: Sized,
    {
        Self {
            parent: Rc::clone(&parent),
            button: Button::new(position, size, parent, |_| (), text),
            drag_start: None,
            start_callback,
            moved_callback,
            released_callback,
        }
    }

    pub fn button<'b>(&'b self) -> &'b Button<'a, T>
    where
        'a: 'b,
    {
        &self.button
    }

    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        println!("Input start");
        if self.button.local_hovers(position) {
            println!("Input on draggable");
            self.drag_start = Some(position);
            let callback = self.start_callback;
            callback(position, state);
        }
    }

    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        if let Some(start) = self.drag_start {
            let callback = self.released_callback;
            callback(start, position, state);
            self.drag_start = None;
        }
    }

    /// TODO: should this button keep track of the exact path which the mouse follows?
    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        if let Some(start) = self.drag_start {
            let callback = self.moved_callback;
            callback(start, position, movement, state);
        }
    }
}
impl<'a, T> Into<UIElement<'a, T>> for DragButton<'a, T> {
    fn into(self) -> UIElement<'a, T> {
        UIElement::DragButton(self)
    }
}
