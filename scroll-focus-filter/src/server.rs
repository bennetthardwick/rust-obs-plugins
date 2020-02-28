use std::rc::Rc;
use xcb_util::{
    ewmh::{get_active_window, get_wm_name, Connection},
    icccm::get_wm_class,
};

#[derive(Debug)]
pub struct WindowSnapshot {
    name: String,
    class: String,
    role: Option<String>,
}

pub struct ActiveWindow {
    connection: Rc<Connection>,
    screen: i32,
    pub window: xcb::Window,
}

impl ActiveWindow {
    fn new(connection: Rc<Connection>, screen: i32) -> ActiveWindow {
        let window =
            Self::get_active_window(&connection, screen).expect("Could not get active window");

        let active_window = Self {
            window,
            connection,
            screen,
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
        let name = get_wm_name(&self.connection, self.window).get_reply()?;
        let class = get_wm_class(&self.connection, self.window).get_reply()?;

        Ok(WindowSnapshot {
            name: String::from(name.string()),
            class: String::from(class.class()),
            role: None,
        })
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

        Ok(Server {
            active: ActiveWindow::new(Rc::clone(&connection), default_screen),
            connection,
        })
    }

    pub fn wait_for_event(&mut self) {
        loop {
            let event = self.connection.wait_for_event().unwrap();
            match event.response_type() {
                xcb::PROPERTY_NOTIFY => {
                    self.active.update();
                    println!("Snapshot: {:?}", self.active.snapshot());
                    return;
                }
                code => {
                    println!("Unknown event! {}", code);
                }
            };
        }
    }
}
