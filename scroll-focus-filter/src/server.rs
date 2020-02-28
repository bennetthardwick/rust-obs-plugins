use std::rc::Rc;
use xcb_util::{
    ewmh::{get_active_window, get_wm_name, Connection},
    icccm::get_wm_class,
};

use xcb::{get_geometry, translate_coordinates};

#[derive(Debug)]
pub struct WindowSnapshot {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub root_width: f32,
    pub root_height: f32,
}

pub struct ActiveWindow {
    connection: Rc<Connection>,
    screen: i32,
    pub window: xcb::Window,
    root_width: f32,
    root_height: f32,
}

impl ActiveWindow {
    fn new(
        connection: Rc<Connection>,
        screen: i32,
        root_width: f32,
        root_height: f32,
    ) -> ActiveWindow {
        let window =
            Self::get_active_window(&connection, screen).expect("Could not get active window");

        let active_window = Self {
            window,
            connection,
            screen,
            root_width,
            root_height,
        };
        active_window.start_listening();

        active_window
    }

    fn get_active_window(connection: &Connection, screen: i32) -> Option<xcb::Window> {
        let active = get_active_window(&connection, screen);
        active.get_reply().ok()
    }

    fn stop_listening(&self) {
        let mask = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
        xcb::change_window_attributes_checked(&self.connection, self.window, &mask)
            .request_check()
            .unwrap();
    }

    fn start_listening(&self) {
        let mask = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)];
        xcb::change_window_attributes_checked(&self.connection, self.window, &mask)
            .request_check()
            .unwrap();
    }

    fn update(&mut self) {
        let window = Self::get_active_window(&self.connection, self.screen)
            .expect("Could not get active window");

        if self.window != window {
            self.stop_listening();
            self.window = window;
            self.start_listening();
        }
    }

    fn snapshot(&self) -> Result<WindowSnapshot> {
        let geom = get_geometry(&self.connection, self.window).get_reply()?;

        let root_geom = get_geometry(&self.connection, geom.root()).get_reply()?;

        let diff = translate_coordinates(
            &self.connection,
            self.window,
            geom.root(),
            geom.x(),
            geom.y(),
        )
        .get_reply()?;

        let snap = WindowSnapshot {
            x: diff.dst_x() as f32,
            y: diff.dst_y() as f32,
            width: geom.width() as f32,
            height: geom.height() as f32,
            root_width: self.root_width,
            root_height: self.root_height,
        };

        println!("Snap {:?}", snap);

        Ok(snap)
    }
}

impl Drop for ActiveWindow {
    fn drop(&mut self) {
        self.stop_listening();
    }
}

pub struct Server {
    connection: Rc<Connection>,
    active: ActiveWindow,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl Server {
    pub fn new() -> Result<Server> {
        let (connection, default_screen) = xcb::Connection::connect(None)?;

        let connection = xcb_util::ewmh::Connection::connect(connection)
            .map_err(|(_a, _b)| "Could not create ewmh connection")?;

        let connection = Rc::new(connection);

        let screen = connection
            .get_setup()
            .roots()
            .nth(default_screen as usize)
            .unwrap();

        Ok(Server {
            active: ActiveWindow::new(
                Rc::clone(&connection),
                default_screen,
                screen.width_in_pixels() as f32,
                screen.height_in_pixels() as f32,
            ),
            connection,
        })
    }

    pub fn wait_for_event(&mut self) -> Option<WindowSnapshot> {
        let event = self.connection.wait_for_event().unwrap();
        match event.response_type() {
            xcb::PROPERTY_NOTIFY => {
                self.active.update();
                self.active.snapshot().ok()
            }
            _ => None,
        }
    }
}
