use env_logger;
use gtk::atk::Window;
use libc::{c_int, c_uchar, c_ulong};
use log::{error, info, warn};
use penrose::pure::Position;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use x11::keysym::XK_F4;
use x11::xlib::{
    self, XConfigureEvent, XConfigureRequestEvent, XCreateWindowEvent, XDestroyWindowEvent,
    XGetGeometry, XMapEvent, XMapRequestEvent, XReparentEvent, XUnmapEvent,
};
use x11::xlib::{
    BadAccess, Display, Mod1Mask, MotionNotify, SubstructureNotifyMask, SubstructureRedirectMask,
    XButtonEvent, XCheckTypedWindowEvent, XDisplayString, XErrorEvent, XEvent, XGetErrorText,
    XKeyEvent, XKeysymToKeycode, XMotionEvent, XNextEvent, XSelectInput, XSetErrorHandler, XSync,
};
pub struct Xwm {
    // Handle to the underlying Xlib Display struct.
    display: *mut xlib::Display,
    // Handle to root window.
    root: xlib::Window,
    clients: HashMap<Window, Window>,
}

//modifying this value could cause undefined behavior, which
//is why unsafe {} blocks are necessary, when assigning a new value
static mut WM_DETECTED: bool = false;

impl Xwm {
    //This method connects to the X server and creates a wm instance
    pub fn create() -> Result<Rc<Self>, String> {
        unsafe {
            //Open X Display or returning Error in case this fails
            let display: *mut Display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                let display_name: CString = CString::new(xlib::XDisplayName(ptr::null())).unwrap();
                return Err(format!(
                    "Error whilst attempting to open an display {}",
                    display_name.to_string_lossy()
                ));
            }

            //Crate a window manager instance by creating a new reference-counting pointer
            Ok(Rc::new(Self::xwm(display)))
        }
    }

    // Invoked internally by create().
    fn xwm(display: *mut xlib::Display) -> Self {
        let clients: HashMap<Window, Window> = HashMap::new();
        return Xwm {
            display,
            root: unsafe { xlib::XDefaultRootWindow(display) },
            clients,
        };
    }

    // Disconnects from the X server.
    pub fn close(&self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }

    // The entry point to this class. Enters the main event loop.
    pub fn run(&self) {
        //We begin by selecting the events on the root window and by
        //using a special error handler we can exit if another wm is running.
        let error_handler_init: Option<
            unsafe extern "C" fn(*mut Display, *mut XErrorEvent) -> c_int,
        > = Some(on_detected_wm);

        unsafe {
            WM_DETECTED = false;
            XSetErrorHandler(error_handler_init);
            XSelectInput(
                self.display,
                self.root,
                SubstructureNotifyMask | SubstructureRedirectMask,
            );
            XSync(self.display, 0);
            if (WM_DETECTED) {
                error!(
                    "Error, other wm on display {}",
                    XDisplayString(self.display)
                );
                return;
            }
        }
        let error_handler: Option<unsafe extern "C" fn(*mut Display, *mut XErrorEvent) -> c_int> =
            Some(on_x_error);

        //now that we now that no other wm is running, we can use a different
        //error handler.
        unsafe {
            XSetErrorHandler(error_handler);
        }

        loop {
            let mut event: XEvent = Default::default();
            unsafe { XNextEvent(self.display, &mut event) };
            info!("Received event: {:?}", event);

            unsafe {
                match (event.type_) {
                    xlib::KeyPress => Self::on_key_press(&self, event.key),
                    xlib::KeyRelease => Self::on_key_release(&self, event.key),
                    xlib::ButtonPress => Self::on_button_press(&self, event.button),
                    xlib::ButtonRelease => Self::on_button_release(&self, event.button),
                    xlib::MotionNotify => {
                        while XCheckTypedWindowEvent(
                            self.display,
                            event.motion.window,
                            MotionNotify,
                            &mut event,
                        ) {}
                        Self::on_motion_notify(&self, event.motion);
                    }
                    xlib::CreateNotify => Self::on_create_notify(&self, event.create_window),
                    xlib::DestroyNotify => Self::on_destroy_notify(&self, event.destroy_window),
                    xlib::ReparentNotify => Self::on_reparant_notify(&self, event.reparent),
                    xlib::MapNotify => Self::on_map_notify(&self, event.map),
                    xlib::UnmapNotify => Self::on_unmap_notify(&self, event.unmap),
                    xlib::ConfigureNotify => Self::on_configure_notify(&self, event.configure),
                    xlib::MapRequest => Self::on_map_request(&self, event.map_request),
                    xlib::ConfigureRequest => {
                        Self::on_configure_request(&self, event.configure_request)
                    }
                    _ => warn!("Ignored event"),
                }
            }
        }
    }

    // Key Events
    fn on_key_press(&self, e: XKeyEvent) {
        unsafe {
            //  check for alt+f4
            if e.keycode == XKeysymToKeycode(self.display, XK_F4 as u64) as u32
                && (e.state & Mod1Mask as u32) != 0
            { //TODO implement this
            }
        }
    }
    fn on_key_release(&self, e: XKeyEvent) {}

    // Button Events
    fn on_button_press(&self, e: XButtonEvent) {
        let window: u64 = e.window;
        assert!(self.clients.contains_key(&window));
        let frame: u64 = self.clients[&window];

        // 1. Save initial cursor position.
        let drag_start_pos = (e.x_root, e.y_root);

        // 2. Save initial window info.
        let mut returned_root: u64 = 0;
        let mut x = 0;
        let mut y = 0;
        let mut width = 0;
        let mut height = 0;
        let mut border_width = 0;
        let mut depth = 0;
        unsafe {
            assert_eq!(
                XGetGeometry(
                    self.display,
                    frame,
                    &mut returned_root,
                    &mut x,
                    &mut y,
                    &mut width,
                    &mut height,
                    &mut border_width,
                    &mut depth,
                ),
                1
            );
        }
    }
    fn on_button_release(&self, e: XButtonEvent) {}

    // Notification Events
    fn on_motion_notify(&self, e: XMotionEvent) {}
    fn on_create_notify(&self, e: XCreateWindowEvent) {}
    fn on_destroy_notify(&self, e: XDestroyWindowEvent) {}
    fn on_reparant_notify(&self, e: XReparentEvent) {}
    fn on_map_notify(&self, e: XMapEvent) {}
    fn on_unmap_notify(&self, e: XUnmapEvent) {}
    fn on_configure_notify(&self, e: XConfigureEvent) {}

    //Request Events
    fn on_map_request(&self, e: XMapRequestEvent) {}
    fn on_configure_request(&self, e: XConfigureRequestEvent) {}
}

extern "C" fn on_x_error(display: *mut Display, e: *mut XErrorEvent) -> c_int {
    const MAX_ERROR_LENGTH: usize = 1024;
    let mut error_text: [u8; MAX_ERROR_LENGTH] = [0; MAX_ERROR_LENGTH];
    unsafe {
        XGetErrorText(
            display,
            (*e).error_code as c_int,
            error_text.as_mut_ptr() as *mut i8,
            MAX_ERROR_LENGTH as i32,
        );

        error!(
            "Got X Error: {}\n Request: {}\nResource ID: {}",
            (*e).error_code as i32,
            (*e).request_code as i32,
            (*e).resourceid
        );
    }
    return 0;
}

extern "C" fn on_detected_wm(display: *mut Display, e: *mut XErrorEvent) -> c_int {
    //Since the only error code we expect in this error handler, we
    //only need to check for that.
    unsafe {
        assert_eq!((*e).error_code as i32, BadAccess.into());
        WM_DETECTED = true;
    }

    return 0;
}
