use crate::config::Config;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::Level;
use tracing_subscriber::{FmtSubscriber, util::SubscriberInitExt};
use penrose::{core::{
    Config as PConfig,
    State,
    WindowManager,
    bindings::{
        KeyEventHandler,
        parse_keybindings_with_xmodmap
    },
    layout::LayoutStack as ls
}, stack, builtin::{
    layout::{
        messages::{
            IncMain,
            ShrinkMain
        },
        MainAndStack,
        Monocle,
        //build in layout transformers e.g. gaps around windows
        transformers::{
            ReflectHorizontal,
            ReserveTop,
            Gaps
        }
    },
    actions::{
        modify_with,
        send_layout_message
    }
}, extensions::{
    actions::toggle_fullscreen,
    hooks::add_ewmh_hooks
}, Color, Result, x::XConn, x11rb::RustConn, util};
use penrose::builtin::actions::floating::float_focused;
use penrose::builtin::actions::key_handler;
use penrose::builtin::layout::messages::ExpandMain;
use penrose::core::bindings::KeyBindings;

use penrose::core::hooks::StateHook;
use penrose::util::spawn_with_args;


#[derive(Default)]
struct HippoWM {
    top_gaps: u32,
    outer_gaps: u32,
    inner_gaps: u32,
    ratio: f32,
    ratio_step: f32,
    max_main: u32
}

impl HippoWM {

    fn get_layouts(&self) -> ls {
        return stack!(
            MainAndStack::side(self.max_main, self.ratio, self.ratio_step),
            ReflectHorizontal::wrap(MainAndStack::side(
                self.max_main,
                self.ratio,
                self.ratio_step
            )),
            MainAndStack::bottom(self.max_main, self.ratio, self.ratio_step),
            Monocle::boxed()
        ).map(|l| {
            ReserveTop::wrap(
            Gaps::wrap(l, self.outer_gaps, self.inner_gaps), self.top_gaps)
        })
    }

    fn ws_binds(&self, kb: HashMap<String, Box<dyn KeyEventHandler<RustConn>>>)
                    -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
        let mut key_bindings: HashMap<String, Box<dyn KeyEventHandler<RustConn>>> = Default::default();
        key_bindings.extend(kb);
        for ws in &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"] {
            key_bindings.extend([
                (format!("M-{}", ws),
                 modify_with(move |client_set| client_set.focus_tag(ws)),),
                (format!("M-S-{}", ws),
                 modify_with(move |client_set| client_set.move_focused_to_tag(ws)))
            ])
        }
        return key_bindings;
    }
    fn configure(&mut self, config: Config) -> HashMap<String, Box<dyn KeyEventHandler<RustConn>>> {
        // set variables
        self.max_main = config.max_main;
        self.top_gaps = config.top_gaps;
        self.inner_gaps = config.inner_gaps;
        self.outer_gaps = config.outer_gap;
        self.ratio = config.ratio;
        self.ratio_step = config.ratio_steps;

        // set keybinds to default ones
        let mut kb: HashMap<String, Box<dyn KeyEventHandler<RustConn>>> = Default::default();

        // set action keybinds
        for a in config.actions {
            if let Some(action) = self.action(&a.action) {
                kb.insert(a.bind, action);
            }
        }
        //set xaction keybinds
        for a in config.x_actions {
            if let Some(action) = self.action(&a.action) {
                kb.insert(a.bind, action);
            }
        }

        //set waction keybidns
        for a in config.window_actions {
            if let Some(action) = self.action(&a.action) {
                kb.insert(a.bind, action);
            }
        }
        //set command keybinds
        for cmd in config.commands {
            kb.insert(cmd.bind, key_handler(move |_, _| util::spawn(cmd.command.as_str())));
        }
        //set xcommand keybinds
        for cmd in config.x_command {
            kb.insert(cmd.bind, key_handler(move |_, _| util::spawn(cmd.command.as_str())));
        }
        //set windowcommand keybidnds
        for cmd in config.window_commands {
            kb.insert(cmd.bind, key_handler(move |_, _| util::spawn(cmd.command.as_str())));
        }

        return kb

    }

    fn action(&self, action: &str) -> Option<Box<dyn KeyEventHandler<RustConn>>> {
        match action.to_lowercase().as_str() {
            "kill" => Some(modify_with(|a| a.kill_focused())),
            "focusnext" => Some(modify_with(|a| a.focus_down())), //focus element down the stack
            "focusprevious" => Some(modify_with(|a| a.focus_up())), //focus element up the stack
            "togglefullscreen" => Some(toggle_fullscreen()),
            "swapup" => Some(modify_with(|a| a.swap_up())),
            "swapdown" => Some(modify_with(|a| a.swap_down())),
            "floatfocused" => Some(float_focused()),
            "toggletag" => Some(modify_with(|a| a.toggle_tag())),
            "nextlayout" => Some(modify_with(|a| a.next_layout())),
            "previouslayout" => Some(modify_with(|a| a.previous_layout())),
            "incmain" => Some(send_layout_message(|| IncMain(1))),
            "decmain" => Some(send_layout_message(|| IncMain(-1))),
            "expandmain" => Some(send_layout_message(|| ExpandMain)),
            "shrmain" => Some(send_layout_message(|| ShrinkMain)),
            _ => None
        }
    }
}
pub fn run(config: Config) -> Result<()>{
    FmtSubscriber::builder().with_max_level(Level::TRACE).finish().init();

    let mut pre_hook : String = "".into();

    if !config.auto_start.is_empty() {
        for (e, i) in config.auto_start.iter().enumerate(){
            pre_hook.push_str(i);
            if config.auto_start.len() > 1 && e + 1 < config.auto_start.len() {
                pre_hook.push_str(" && ");
            }
        }
    }

    let startup_hook: Option<Box<dyn StateHook<RustConn>>> = if pre_hook.is_empty() {
        None
    } else {
        Some(SpawnOnStartup::make_box(pre_hook))
    };

    let mut hippowm: HippoWM = HippoWM::default();
    let kb : HashMap<String, Box<dyn KeyEventHandler<RustConn>>> = hippowm.configure(config.clone());

    let conf = add_ewmh_hooks(PConfig {
        default_layouts: hippowm.get_layouts(),
        normal_border: Color::new_from_hex(config.border),
        focused_border: Color::new_from_hex(config.focused_border),
        startup_hook,
        tags: config.workspaces,
        ..PConfig::default()
    });

    let keys : KeyBindings<RustConn> = parse_keybindings_with_xmodmap(hippowm.ws_binds(kb))?;
    let rustc = RustConn::new()?;
    let wm = WindowManager::new(
        conf,
        keys,
        HashMap::new(),
        rustc
    )?;

    wm.run().unwrap();
    Ok(())
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct SpawnOnStartup {
    //A clone-on-write smart pointer,
    pointer: Cow<'static, str>,
}

impl SpawnOnStartup {
    pub fn make_box<T> (pointer: impl Into<Cow<'static, str>>) -> Box<dyn StateHook<T>>
    where T: XConn,
    {
        return Box::new(Self {pointer: pointer.into()})
    }
}

impl<T> StateHook<T> for SpawnOnStartup where T: XConn {
    fn call(&mut self, _state: &mut State<T>, _x: &T) -> Result<()> {
        let arguments:[&str; 2] = ["-c", &self.pointer];
        spawn_with_args("bash", &arguments)?;
        Ok(())
    }
}