use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub max_main: u32,
    pub border: u32,
    pub focused_border: u32,
    pub ratio: f32,
    pub ratio_steps: f32,
    pub inner_gaps: u32,
    pub outer_gap: u32,
    pub top_gaps: u32,
    pub commands: Vec<Command>,
    pub window_commands: Vec<Command>,
    pub x_command: Vec<Command>,
    pub actions: Vec<Action>,
    pub window_actions: Vec<Action>,
    pub x_actions: Vec<Action>,
    pub workspaces: Vec<String>,
    pub auto_start: Vec<String>,


}

impl Default for Config {
    fn default() -> Self {
        let mut workspaces: Vec<String> = vec![];
        for ws in 1..10 {
            workspaces.push(ws.to_string().into());
        }
        workspaces.push("0".into());
        let config = Config {
            max_main : 1,
            border: 0x00000000,
            focused_border: 0xf00e70ef,
            ratio: 0.5,
            ratio_steps: 0.0,
            inner_gaps: 0,
            outer_gap: 0,
            top_gaps: 0,
            commands: get_commands(),
            window_commands: vec![],
            x_command: vec![],
            actions: vec![
                Action {
                    bind: "M-S-q".into(),
                    action: "kill".into()
                }
            ],
            window_actions: vec![],
            x_actions: vec![
                Action {
                    bind: "M-l".into(),
                    action: "focusNext".into(),
                },
                Action {
                    bind: "M-h".into(),
                    action: "focusPrevious".into(),
                },
                Action {
                    bind: "M-f".into(),
                    action: "ToggleFullScreen".into(),
                },
                Action {
                    bind: "M-S-k".into(),
                    action: "SwapUp".into(),
                },
                Action {
                    bind: "M-S-j".into(),
                    action: "SwapDown".into(),
                },
                Action {
                    bind: "M-S-f".into(),
                    action: "floatfocused".into(),
                },
                Action {
                    bind: "M-Tab".into(),
                    action: "ToggleTag".into(),
                },
                Action {
                    bind: "M-bracketright".into(),
                    action: "FocusNextScreen".into(),
                },
                Action {
                    bind: "M-bracketleft".into(),
                    action: "FocusPreviousScreen".into(),
                },
                Action {
                    bind: "M-grave".into(),
                    action: "NextLayout".into(),
                },
                Action {
                    bind: "M-S-grave".into(),
                    action: "PreviousLayout".into(),
                },
                Action {
                    bind: "M-S-Up".into(),
                    action: "IncMain".into(),
                },
                Action {
                    bind: "M-S-Down".into(),
                    action: "DecMain".into(),
                },
                Action {
                    bind: "M-S-Right".into(),
                    action: "ExpMain".into(),
                },
                Action {
                    bind: "M-S-Up".into(),
                    action: "ShrMain".into(),
                },
            ],
            workspaces,
            auto_start: vec![],
        };
        confy::store("hippowm", Some("config"), config).unwrap();
        confy::load("hippowm", Some("config")).unwrap()
    }

}
pub fn get_commands() -> Vec<Command> {
    return vec![
        Command {
            bind: "M-Return".into(),
            command: "kitty".into(),
        },
        Command {
            bind: "M-p".into(),
            command: "rofi -show drun".into()
        }
    ]
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub bind: String,
    pub action: String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub bind: String,
    pub command: String
}


pub fn get_config() -> Config {
    confy::load("hippowm", Some("config")).unwrap_or_else(|_| {
        confy::store("hippowm", Some("config"), Config::default()).unwrap();
        confy::load("hippowm", Some("config")).unwrap()
    })
}