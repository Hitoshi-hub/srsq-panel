use relm4::prelude::*;

use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};
use gtk4::prelude::*;
use gtk4::glib;

use std::time::Duration;
use std::fs;
use std::path::PathBuf;

struct PowerMenu {
    current_margin: f32,
    target_margin: f32,
    is_closing: bool,
    pic_path: String,
    shutdown_icon: String,
    reboot_icon: String,
    sleep_icon: String,
    exit_icon: String,
}

#[derive(Debug)]
enum Msg {
    Tick,
    Execute(String),
    Close,
}

#[relm4::component]
impl Component for PowerMenu {

    
    type Init = ();
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();
    
    view! {
        gtk4::Window {
            set_resizable: false,

            add_controller = gtk4::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    if key == gtk4::gdk::Key::Escape {
                        println!("Esc detected...");
                        sender.input(Msg::Close);
                        gtk4::glib::Propagation::Stop
                        
                    } else {
                        gtk4::glib::Propagation::Proceed
                    }
                }
            },

            gtk4::Box {
                set_width_request: 100,
                set_height_request: 300,      

                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk4::Align::Fill,
                set_valign: gtk4::Align::Center,

                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 15,
                set_margin_all: 20,




                #[name = "start_button"]
                gtk::Button {
                    set_width_request: 128,
                    set_height_request: 128,

                    set_halign: gtk4::Align::Center,
                    set_valign: gtk4::Align::Center,
                    
                    gtk4::Image{
                        set_from_file: Some(&model.shutdown_icon),
                        set_pixel_size: 64,
                    },
                    connect_clicked => Msg::Execute("shutdown now".to_string()),
                },
                gtk::Button {
                    set_width_request: 128,
                    set_height_request: 128,

                    set_halign: gtk4::Align::Center,
                    set_valign: gtk4::Align::Center,

                    gtk4::Image{
                        set_from_file: Some(&model.reboot_icon),
                        set_pixel_size: 64,
                    },
                    connect_clicked => Msg::Execute("reboot".to_string()),
                },

                gtk4::Image {
                    add_css_class: "menu-gif",
    
                    set_from_file: Some(&model.pic_path),
                    set_pixel_size: 128,

                },

                gtk::Button {
                    set_width_request: 128,
                    set_height_request: 128,

                    set_halign: gtk4::Align::Center,
                    set_valign: gtk4::Align::Center,

                    gtk4::Image{
                        set_from_file: Some(&model.sleep_icon),
                        set_pixel_size: 64,
                    },
                    
                    connect_clicked => Msg::Execute("systemctl suspend".to_string()),
                },
                gtk::Button {
                    set_width_request: 128,
                    set_height_request: 128,

                    set_halign: gtk4::Align::Center,
                    set_valign: gtk4::Align::Center,
                    
                    gtk4::Image{
                        set_from_file: Some(&model.exit_icon),
                        set_pixel_size: 64,
                    },
                    connect_clicked => Msg::Execute("exit".to_string()),
                },
            
            }
        }
    }


    fn init(
        _init: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        let initial_margin = -150;
        window.set_margin(Edge::Right, initial_margin);

        let home = std::env::var("HOME").unwrap_or_default();
        let config_path = format!("{}/.config/srsq-panel", home);

        let get_path = |name: &str| format!("{}/{}", config_path, name);

        let model = PowerMenu{
            current_margin: initial_margin as f32,
            target_margin: 0.0,
            is_closing: false,

            pic_path: get_path("picture.png"),
            shutdown_icon: get_path("shutdown.png"),
            reboot_icon: get_path("reboot.png"),
            sleep_icon: get_path("sleep.png"),
            exit_icon: get_path("exit.png"),
        };



        let tick_sender = sender.clone();

        glib::timeout_add_local(Duration::from_millis(16), move || {
            tick_sender.input(Msg::Tick);
            glib::ControlFlow::Continue
        });

        let widgets = view_output!();
        widgets.start_button.grab_focus();

        ComponentParts {model, widgets}
    }

    fn update(
        &mut self, 
        msg: Self::Input, 
        _sender: ComponentSender<Self>, 
        root: &Self::Root
    ) {
        match msg {
            Msg::Tick => {
                let target = if self.is_closing { -200.0 } else { self.target_margin };

                let diff = (target - self.current_margin) as f32;

                if diff.abs() > 0.1 {
                    let step = diff * 0.2;
                    let smooth_step = if  diff > 0.0 {step.ceil()} else {step.floor()};
                    self.current_margin += smooth_step;
                    root.set_margin(Edge::Right, self.current_margin as i32);
                }
                if self.is_closing && self.current_margin <= -199.0 {
                    println!("Closing panel");
                    relm4::main_application().quit();
                }
            }
            Msg::Execute(cmd) => {
                println!("Executing {}", cmd);
                std::process::Command::new("sh").arg("-c").arg(cmd).spawn().ok();
                self.is_closing = true;
            }
            Msg::Close => { self.is_closing = true; }
        }  
    }
}

fn load_css() {
    let config_dir = std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".config/srsq-panel"))
        .expect("Unable to find home directory");

    let _ = fs::create_dir_all(&config_dir);
    let css_path = config_dir.join("style.css");
    let file = gtk4::gio::File::for_path(&css_path);

    let provider = gtk4::CssProvider::new();

    if css_path.exists() {
        provider.load_from_file(&file);
        println!("CSS config was loaded");
    } else {
        let default_css = "
        window { 
                background-color: rgba(30, 30, 46, 1); 
                border-radius: 20px 0 0 20px; 
                border: 1px solid #000000;
            }
            button { 
                background: #313244; 
                color: #cdd6f4; 
                margin: 5px; 
                padding: 10px 20px; 
                border-radius: 10px; 
                border: none;
            }
            button:focus { 
                background: #89b4fa; 
                color: #11111b; 
            }
        ";
        provider.load_from_data(default_css);
        println!("CSS config has not found, using default configuration");

    }
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Display has not found"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}


fn main() {
    unsafe{std::env::set_var("GDK_BACKEND", "wayland");}
    let app = RelmApp::new("com.hitoshi.srsqpowermenu");
    load_css();
    app.run::<PowerMenu>(());
}