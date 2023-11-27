use std::{cell::RefCell, rc::Rc};

use ggez::{
    graphics::{self, Color},
    Context,
};

use crate::{
    renderer::{draw_rectangle, draw_text},
    vec2d,
    vector::Vector,
};

/// Represents a part of the UI
pub enum UIElement<'a, T> {
    Button(Button<'a, T>),
    Menu(Menu<'a, T>),
    DragButton(DragButton<'a, T>),
}

impl<T> UIElement<'_, T> {
    /// Gets the position of the element
    pub fn position(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.x(), x.y()),
            UIElement::Menu(x) => x.position(),
            UIElement::DragButton(x) => vec2d![x.button().x(), x.button().y()],
        }
    }

    /// Gets the width and height of the element
    pub fn size(&self) -> Vector {
        match self {
            UIElement::Button(x) => vec2d!(x.width(), x.height()),
            UIElement::Menu(x) => x.size(),
            UIElement::DragButton(x) => vec2d![x.button().width(), x.button().height()],
        }
    }

    /// Gets the size of an element, ignoring any scaling
    /// by the parent element(s)
    pub fn local_size(&self) -> Vector {
        match self {
            UIElement::Button(x) => x.size,
            UIElement::Menu(x) => x.local_size(),
            UIElement::DragButton(x) => x.button.size,
        }
    }

    /// Gets the position of an element, ignoring any translation
    /// by the parent element(s)
    pub fn local_position(&self) -> Vector {
        match self {
            UIElement::Button(x) => x.position,
            UIElement::Menu(x) => x.position,
            UIElement::DragButton(x) => x.button.position,
        }
    }

    /// Draws the element to the canvas
    pub fn draw(&self, ctx: &mut Context) {
        match self {
            UIElement::Button(x) => x.draw(ctx),
            UIElement::Menu(x) => x.draw(ctx),
            UIElement::DragButton(x) => x.button().draw(ctx),
        }
    }

    /// Tells this element that the mouse was pressed at a given
    /// position
    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        match self {
            UIElement::Menu(x) => x.input_start(position, state),
            UIElement::DragButton(x) => x.input_start(position, state),
            _ => (),
        }
    }

    /// Tells this element that the mouse was moved, while held down,
    /// by `movement` to `position`
    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        match self {
            UIElement::Menu(x) => x.input_moved(position, movement, state),
            UIElement::DragButton(x) => x.input_moved(position, movement, state),
            _ => (),
        }
    }

    /// Tells this element that the mouse was released at `position`
    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        match self {
            UIElement::Button(x) => x.input_released(position, state),
            UIElement::Menu(x) => x.input_released(position, state),
            UIElement::DragButton(x) => x.input_released(position, state),
        }
    }
}

/// Represents a button with text, which can be clicked
pub struct Button<'a, T> {
    parent: Rc<RefCell<Menu<'a, T>>>,
    position: Vector,
    size: Vector,
    callback: fn(&mut T),
    text: String,
}

impl<'a, T> Button<'a, T> {
    /// Gets the global x coordinate of this button
    pub fn x(&self) -> f32 {
        self.position.x * self.parent.borrow().scale() + self.parent.borrow().position().x
    }
    /// Gets the global y coordinate of this button
    pub fn y(&self) -> f32 {
        self.position.y * self.parent.borrow().scale() + self.parent.borrow().position().y
    }
    /// Gets the global width of this button
    pub fn width(&self) -> f32 {
        self.size.x * self.parent.borrow().scale()
    }
    /// Gets the global height of this button
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

    /// Checks if the mouse hovers over this button,
    /// given the local position of the mouse (relative to
    /// the `parent` menu)
    pub fn local_hovers(&self, mouse: Vector) -> bool {
        self.position.x <= mouse.x
            && self.position.y <= mouse.y
            && self.position.x + self.size.x >= mouse.x
            && self.position.y + self.size.y >= mouse.y
    }

    /// Responds to this button being clicked
    pub fn click(&self, state: &mut T) {
        let callback = self.callback;
        callback(state);
    }

    /// Draws this button
    pub fn draw(&self, ctx: &mut Context) {
        draw_rectangle(
            ctx,
            vec2d!(self.x(), self.y()),
            vec2d!(self.width(), self.height()),
            Color::from_rgb(200, 200, 200),
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

    /// Responds to the mouse being released,
    /// by `Button::click`ing if the mouse is over the button
    pub fn input_released(&self, position: Vector, state: &mut T) {
        if self.local_hovers(position) {
            self.click(state);
        }
    }
}

impl<'a, T> From<Button<'a, T>> for UIElement<'a, T> {
    fn from(val: Button<'a, T>) -> Self {
        UIElement::Button(val)
    }
}

/// Represents a collection of UI elements
/// which have a common translation/scaling applied to them
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

    /// Adds elements to this collection
    pub fn add_elements(&mut self, elements: Vec<UIElement<'a, T>>) {
        self.elements.extend(elements.into_iter());
    }

    /// Gets the global position of menu
    pub fn position(&self) -> Vector {
        match self.parent {
            Some(parent) => self.position * parent.scale + parent.position(),
            None => self.position,
        }
    }

    /// Gets the global scale of this menu
    pub fn scale(&self) -> f32 {
        match self.parent {
            Some(parent) => self.scale * parent.scale(),
            None => self.scale,
        }
    }

    /// Returns the global (top_left, bottom_right)
    pub fn bounds(&self) -> (Vector, Vector) {
        let initial: Vector = self.elements[0].position();
        let initial: (Vector, Vector) = (initial, initial);
        self.elements.iter().fold(initial, |bounds, element| {
            let position = element.position();
            let size = element.size();
            (bounds.0.min(position), bounds.1.max(position + size))
        })
    }

    /// Returns the local (top_left, buttom_right)
    pub fn local_bounds(&self) -> (Vector, Vector) {
        let initial: Vector = self.elements[0].local_position();
        let initial: (Vector, Vector) = (initial, initial);
        self.elements.iter().fold(initial, |bounds, element| {
            let position = element.local_position();
            let size = element.local_size();
            (bounds.0.min(position), bounds.1.max(position + size))
        })
    }

    /// Gets the global width and height of this menu
    pub fn size(&self) -> Vector {
        let bounds = self.bounds();
        bounds.1 - bounds.0
    }

    /// Gets the local width and height of this menu
    pub fn local_size(&self) -> Vector {
        let bounds = self.local_bounds();
        bounds.1 - bounds.0
    }

    /// Draws this menu
    pub fn draw(&self, ctx: &mut Context) {
        self.elements.iter().for_each(|x| x.draw(ctx));
    }

    /// Gets the local position of this menu
    pub fn local_position(&self) -> Vector {
        self.local_bounds().0
    }

    /// Like `Button::local_hovers`
    pub fn local_hovers(&self, position: Vector) -> bool {
        (position.x >= self.local_position().x)
            && (position.y >= self.local_position().y)
            && (position.x <= self.local_position().x + self.local_size().x)
            && (position.y <= self.local_position().y + self.local_size().y)
    }

    /// Tells all elements in this collection
    /// that the mouse has been released
    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        let local_position = self.position;
        for element in self.elements.iter_mut() {
            element.input_released((position - local_position) / self.scale, state);
        }
    }

    /// Tells all elements in this collection
    /// that the mouse has been moved, while held down
    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        let local_position = self.position;
        for element in self.elements.iter_mut() {
            element.input_moved(
                (position - local_position) / self.scale,
                movement / self.scale,
                state,
            )
        }
    }

    /// Tells all elements in this collection
    /// that the mouse has been held down
    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        if self.local_hovers(position) {
            let local_position = self.position;
            for element in self.elements.iter_mut() {
                element.input_start((position - local_position) / self.scale, state);
            }
        }
    }
}

impl<'a, T> From<Menu<'a, T>> for UIElement<'a, T> {
    fn from(val: Menu<'a, T>) -> Self {
        UIElement::Menu(val)
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

/// A button from which things can be dragged
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

    /// Gets the inner button which this is based off of
    pub fn button<'b>(&'b self) -> &'b Button<'a, T>
    where
        'a: 'b,
    {
        &self.button
    }

    /// Like `Button::input_start`
    pub fn input_start(&mut self, position: Vector, state: &mut T) {
        if self.button.local_hovers(position) {
            self.drag_start = Some(position);
            let callback = self.start_callback;
            callback(position, state);
        }
    }

    /// Like `Button::input_released`
    pub fn input_released(&mut self, position: Vector, state: &mut T) {
        if let Some(start) = self.drag_start {
            let callback = self.released_callback;
            callback(start, position, state);
            self.drag_start = None;
        }
    }

    /// Like `Button::input_moved`
    pub fn input_moved(&mut self, position: Vector, movement: Vector, state: &mut T) {
        if let Some(start) = self.drag_start {
            let callback = self.moved_callback;
            callback(start, position, movement, state);
        }
    }
}
impl<'a, T> From<DragButton<'a, T>> for UIElement<'a, T> {
    fn from(val: DragButton<'a, T>) -> Self {
        UIElement::DragButton(val)
    }
}
