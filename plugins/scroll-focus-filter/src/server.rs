use std::rc::Rc;
use xcb::{get_geometry, translate_coordinates};
use xcb_util::ewmh::{get_active_window, Connection};

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
    root: xcb::Window,
    root_width: f32,
    root_height: f32,
}

impl ActiveWindow {
    fn new(
        connection: Rc<Connection>,
        screen: i32,
        root: xcb::Window,
        root_width: f32,
        root_height: f32,
    ) -> Result<ActiveWindow> {
        let window = Self::get_active_window(&connection, screen).unwrap_or(root);

        let active_window = Self {
            window,
            connection,
            screen,
            root,
            root_width,
            root_height,
        };

        active_window.start_listening()?;

        Ok(active_window)
    }

    fn get_active_window(connection: &Connection, screen: i32) -> Result<xcb::Window> {
        let active = get_active_window(connection, screen);
        Ok(active.get_reply()?)
    }

    fn stop_listening(&self) -> Result<()> {
        if self.window == self.root {
            Ok(())
        } else {
            let mask = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
            xcb::change_window_attributes_checked(&self.connection, self.window, &mask)
                .request_check()?;
            Ok(())
        }
    }

    fn start_listening(&self) -> Result<()> {
        let mask = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_PROPERTY_CHANGE
                | xcb::EVENT_MASK_FOCUS_CHANGE
                | xcb::EVENT_MASK_STRUCTURE_NOTIFY
                | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                | xcb::EVENT_MASK_LEAVE_WINDOW,
        )];
        xcb::change_window_attributes_checked(&self.connection, self.window, &mask)
            .request_check()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        let window = Self::get_active_window(&self.connection, self.screen).unwrap_or(self.root);

        if self.window != window {
            self.stop_listening().unwrap_or(());
            self.window = window;
            self.start_listening()?;
        }

        Ok(())
    }

    fn snapshot(&self) -> Result<WindowSnapshot> {
        let geom = get_geometry(&self.connection, self.window).get_reply()?;

        let _root_geom = get_geometry(&self.connection, geom.root()).get_reply()?;

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

        Ok(snap)
    }
}

impl Drop for ActiveWindow {
    fn drop(&mut self) {
        self.stop_listening().unwrap_or(());
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

        let mask = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY)];
        xcb::change_window_attributes_checked(&connection, screen.root(), &mask).request_check()?;

        Ok(Server {
            active: ActiveWindow::new(
                Rc::clone(&connection),
                default_screen,
                screen.root(),
                screen.width_in_pixels() as f32,
                screen.height_in_pixels() as f32,
            )?,
            connection,
        })
    }

    pub fn wait_for_event(&mut self) -> Option<WindowSnapshot> {
        if self.connection.wait_for_event().is_some() {
            if self.active.update().is_err() {
                None
            } else {
                self.active.snapshot().ok()
            }
        } else {
            None
        }
    }
}
