use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use libc::{c_int, c_uchar};
use x11::xlib;
use x11::xlib::{BadAccess, Display, MotionNotify, SubstructureNotifyMask, SubstructureRedirectMask, XCheckTypedWindowEvent, XDisplayString, XErrorEvent, XEvent, XGetErrorText, XNextEvent, XSelectInput, XSetErrorHandler, XSync};
use log::{error, info, warn};
use env_logger;
pub struct Xwm {
    // Handle to the underlying Xlib Display struct.
    display: *mut xlib::Display,
    // Handle to root window.
    root: xlib::Window,
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
                let display_name: CString = CString::new(
                    xlib::XDisplayName(ptr::null())).unwrap();
                return Err(format!("Error whilst attempting to open an display {}", display_name.to_string_lossy()));
            }

            //Crate a window manager instance by creating a new reference-counting pointer
            Ok(Rc::new(Self::xwm(display)))
        }
    }

    // Invoked internally by create().
    fn xwm(display: *mut xlib::Display) -> Self {
        return Xwm {
            display,
            root: unsafe { xlib::XDefaultRootWindow(display) },
        }
    }

    // Disconnects from the X server.
    pub fn close(&self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }

    pub fn on_x_error(display: *mut Display, e: *mut XErrorEvent) -> i32 {
        const MAX_ERROR_LENGTH: usize = 1024;
        let mut error_text: [u8; MAX_ERROR_LENGTH] = [0;MAX_ERROR_LENGTH];
        unsafe {
            XGetErrorText(display, (*e).error_code as c_int, error_text.as_mut_ptr() as *mut i8,
                          MAX_ERROR_LENGTH as i32);

            error!("Got X Error: {}\n Request: {}\nResource ID: {}",
                (*e).error_code as i32,
                (*e).request_code as i32, (*e).resourceid);
        }
        return 0;
    }

    pub fn on_detected_wm(display: *mut Display, e: *mut XErrorEvent) -> i32 {
        //Since the only error code we expect in this error handler, we
        //only need to check for that.
        unsafe {
            assert_eq!((*e).error_code as i32, BadAccess);
            WM_DETECTED = true;
        }

        return 0;

    }

    // The entry point to this class. Enters the main event loop.
    pub fn run(&self) {
        //We begin by selecting the events on the root window and by
        //using a special error handler we can exit if another wm is running.
        let error_handler_init: Option<unsafe fn(*mut Display, *mut XErrorEvent) -> c_int> = Some(Self::on_detected_wm);

        unsafe {
            WM_DETECTED = false;
            XSetErrorHandler(error_handler_init);
            XSelectInput(self.display, self.root, SubstructureNotifyMask | SubstructureRedirectMask);
            XSync(self.display, 0);
            if (WM_DETECTED) {
                error!("Error, other wm on display {}", XDisplayString(self.display));
                return;
            }
        }
        let error_handler: Option<unsafe fn(*mut Display, *mut XErrorEvent) -> c_int> = Some(Self::on_detected_wm);

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
                match(event.type_) {
                    xlib::KeyPress => Self::on_key_press(event.key),
                    xlib::KeyRelease => Self::on_key_release(event.key),
                    xlib::ButtonPress => Self::on_button_press(event.button),
                    xlib::ButtonRelease => Self::on_button_release(event.button),
                    xlib::MotionNotify => {
                        while XCheckTypedWindowEvent(self.display, event.motion.window, MotionNotify, &mut event) {
                        }
                        Self::on_motion_notify(event.motion);
                    }
                    xlib::CreateNotify => Self::on_create_notify(event.create_window),
                    xlib::DestroyNotify => Self::on_destroy_notify(event.destroy_window),
                    xlib::ReparentNotify => Self::on_reparant_notify(event.reparent),
                    xlib::MapNotify => Self::on_map_notify(event.map),
                    xlib::UnmapNotify => Self::on_unmap_notify(event.unmap),
                    xlib::ConfigureNotify => Self::on_configure_notify(event.configure),
                    xlib::MapRequest => Self::on_map_requenst(event.map_request),
                    xlib::ConfigureRequest => Self::on_configure_request(event.configure_request),
                    _ => warn!("Ignored event")
                }
            }
        }
    }
}