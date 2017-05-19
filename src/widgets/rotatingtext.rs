use std::time::{Duration, Instant};
use widget::{State, I3BarWidget};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct RotatingTextWidget {
    rotation_pos: usize,
    width: usize,
    rotation_interval: Duration,
    rotation_speed: Duration,
    next_rotation: Option<Instant>,
    content: String,
    icon: Option<String>,
    state: State,
    rendered: Value,
    cached_output: Option<String>,
    theme: Value,
    pub rotating: bool
}


impl RotatingTextWidget {
    pub fn new(interval: Duration, speed: Duration, width: usize, theme: Value) -> RotatingTextWidget {
        RotatingTextWidget {
            rotation_pos: 0,
            width: width,
            rotation_interval: interval,
            rotation_speed: speed,
            next_rotation: None,
            content: String::new(),
            icon: None,
            state: State::Idle,
            rendered: json!({
                "full_text": "",
                "separator": false,
                "separator_block_width": 0,
                "background": "#000000",
                "color": "#000000"
            }),
            cached_output: None,
            theme: theme,
            rotating: false,
        }
    }

    pub fn with_icon(mut self, name: &str) -> Self {
        self.icon = Some(String::from(self.theme["icons"][name].as_str().expect("Wrong icon identifier!")));
        self.update();
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self.update();
        self
    }

    pub fn with_text(mut self, content: &str) -> Self {
        self.content = String::from(content);
        self.rotation_pos = 0;
        if self.content.len() > self.width {
            self.next_rotation = Some(Instant::now() + self.rotation_interval);
        } else {
            self.next_rotation = None;
        }
        self.update();
        self
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        self.update();
    }

    pub fn set_icon(&mut self, name: &str) {
        self.icon = Some(String::from(self.theme["icons"][name].as_str().expect("Wrong icon identifier!")));
        self.update();
    }

    pub fn set_text(&mut self, content: String) {
        if self.content != content{
            self.content = content;
            self.rotation_pos = 0;
            if self.content.len() > self.width {
                self.next_rotation = Some(Instant::now() + self.rotation_interval);
            } else {
                self.next_rotation = None;
            }
        }
        self.update();
    }

    fn get_rotated_content(&self) -> String {
        if self.content.len() > self.width {
            let missing = (self.rotation_pos + self.width).saturating_sub(self.content.len());
            if missing == 0 {
                self.content.chars().skip(self.rotation_pos).take(self.width).collect()
            } else {
                let mut avail: String = self.content.chars().skip(self.rotation_pos).take(self.width).collect();
                avail.push_str("|");
                avail.push_str(&self.content.chars().take(missing - 1).collect::<String>());
                avail
            }
            
        } else {
            self.content.clone()
        }
    }

    fn update(&mut self) {
        let (key_bg, key_fg) = self.state.theme_keys();

        self.rendered = json!({
            "full_text": format!("{}{} ",
                                self.icon.clone().unwrap_or(String::from(" ")),
                                self.get_rotated_content()),
            "separator": false,
            "separator_block_width": 0,
            "min_width": if self.content == "" {0} else {240},
            "align": "left",
            "background": self.theme[key_bg],
            "color": self.theme[key_fg]
        });

        self.cached_output = Some(self.rendered.to_string());
    }

    pub fn next(&mut self) -> (bool, Option<Duration>) {
        if let Some(next_rotation) = self.next_rotation {
            if next_rotation > Instant::now() {
                (false, Some(next_rotation - Instant::now()))
            } else {
                if self.rotating {
                    if self.rotation_pos < self.content.len() {
                        self.rotation_pos += 1;
                        self.next_rotation = Some(Instant::now() + self.rotation_speed);
                        self.update();
                        (true, Some(self.rotation_speed))
                    } else {
                        self.rotation_pos = 0;
                        self.rotating = false;
                        self.next_rotation = Some(Instant::now() + self.rotation_interval);
                        self.update();
                        (true, Some(self.rotation_interval))
                    }
                } else {
                    self.rotating = true;
                    (true, Some(self.rotation_speed))
                }
            }
        } else {
            (false, None)
        }
    }
}

impl I3BarWidget for RotatingTextWidget {
    fn to_string(&self) -> String {
        self.cached_output.clone().unwrap_or(self.rendered.to_string())
    }

    fn get_rendered(&self) -> &Value {
        &self.rendered
    }
}