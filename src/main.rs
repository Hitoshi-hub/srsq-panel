use relm4::prelude::*;

use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};
use gtk4::prelude::*;
use gtk4::glib;

use std::time::Duration;
use std::fs;
use std::path::PathBuf;
use std::os::unix::net::{UnixListener, UnixStream};

struct PowerMenu {
    bg_opacity: f32,
    target_opacity: f32,
    panel_revealed: bool,
    is_closing: bool,
    anim_stream: gtk4::MediaFile,
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

            #[name = "overlay"]
            gtk4::Overlay {
                
                // --- Фоновый слой (на весь экран) ---
                #[name = "bg_box"]
                gtk4::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    
                    set_valign: gtk4::Align::Fill,
                    set_halign: gtk4::Align::Fill,

                    add_css_class: "fullscreen-bg",
                    #[watch]
                    set_opacity: model.bg_opacity as f64,
                    add_controller = gtk4::GestureClick {
                        set_button: gtk4::gdk::BUTTON_PRIMARY,
                        connect_pressed[sender] => move |_, _n_press, _x,_y| {
                            println!("Background click detected...");
                            sender.input(Msg::Close);
                        }
                    },
                },

                // --- Слой с меню (справа) ---
                add_overlay = &gtk4::Box {
                    set_halign: gtk4::Align::End,
                    set_valign: gtk4::Align::Fill,

                    // Revealer отвечает за плавное выезжание из-за границы виджета
                    #[name = "panel_revealer"]
                    gtk4::Revealer {
                        set_transition_type: gtk4::RevealerTransitionType::SlideLeft,
                        set_transition_duration: 350, // Длительность в мс
                        #[watch]
                        set_reveal_child: model.panel_revealed,

                        // Сама панель меню
                        gtk4::Box {
                            add_css_class: "panel-box",
                            set_width_request: 120,
                            set_orientation: gtk4::Orientation::Vertical,
                            set_valign: gtk4::Align::Center,
                            set_halign: gtk4::Align::End,

                            // Контейнер для центрирования кнопок по вертикали
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_valign: gtk4::Align::Center,
                                set_vexpand: true,
                                set_spacing: 15,
                                set_margin_all: 20,

                                #[name = "start_button"]
                                gtk4::Button {
                                    set_width_request: 128,
                                    set_height_request: 128,
                                    set_halign: gtk4::Align::Center,
                                    set_valign: gtk4::Align::Center,
                                    
                                    gtk4::Image {
                                        set_from_file: Some(&model.shutdown_icon),
                                        set_pixel_size: 64,
                                    },
                                    connect_clicked => Msg::Execute("shutdown now".to_string()),
                                },
                                gtk4::Button {
                                    set_width_request: 128,
                                    set_height_request: 128,
                                    set_halign: gtk4::Align::Center,
                                    set_valign: gtk4::Align::Center,

                                    gtk4::Image {
                                        set_from_file: Some(&model.reboot_icon),
                                        set_pixel_size: 64,
                                    },
                                    connect_clicked => Msg::Execute("reboot".to_string()),
                                },

                                gtk4::Image {
                                    add_css_class: "menu-gif",
                                    set_paintable: Some(&model.anim_stream),
                                    set_halign: gtk4::Align::Center,
                                    set_valign: gtk4::Align::Center,
                                    set_pixel_size: 128,
                                    
                                },

                                gtk4::Button {
                                    set_width_request: 128,
                                    set_height_request: 128,
                                    set_halign: gtk4::Align::Center,
                                    set_valign: gtk4::Align::Center,

                                    gtk4::Image {
                                        set_from_file: Some(&model.sleep_icon),
                                        set_pixel_size: 64,
                                    },
                                    
                                    connect_clicked => Msg::Execute("systemctl suspend".to_string()),
                                },
                                gtk4::Button {
                                    set_width_request: 128,
                                    set_height_request: 128,
                                    set_halign: gtk4::Align::Center,
                                    set_valign: gtk4::Align::Center,
                                    
                                    gtk4::Image {
                                        set_from_file: Some(&model.exit_icon),
                                        set_pixel_size: 64,
                                    },
                                    connect_clicked => Msg::Execute("loginctl terminate-user $USER".to_string()),
                                },
                            }
                        }
                    }
                }
            }
        }
    }



    fn init(
        _init: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        
        let socket_sender = sender.clone();
        std::thread::spawn(move || {
            let socket_path = "/tmp/srsq-panel.sock";
            let _ = std::fs::remove_file(socket_path);

            if let Ok(listener) = UnixListener::bind(socket_path){
                for stream in listener.incoming() {
                    if let Ok(_) = stream {
                        socket_sender.input(Msg::Close);
                    }
                }
            }
        });

        // Настройки LayerShell: растягиваем окно на все края
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        let home = std::env::var("HOME").unwrap_or_default();
        let config_path = format!("{}/.config/srsq-panel", home);
        let get_path = |name: &str| format!("{}/{}", config_path, name);

        let gif_path = get_path("picture.gif");
        let file = gtk4::gio::File::for_path(&gif_path);

        let anim_stream = gtk4::MediaFile::for_file(&file);

        anim_stream.set_loop(true);
        anim_stream.play();

        let model = PowerMenu {
            bg_opacity: 0.0,
            target_opacity: 1.0,
            panel_revealed: false,
            is_closing: false,

            anim_stream,
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
        _root: &Self::Root
    ) {
        match msg {
            Msg::Tick => {
                // Плавное изменение прозрачности фона
                let target = if self.is_closing { 0.0 } else { self.target_opacity };
                let diff = target - self.bg_opacity;

                if diff.abs() > 0.01 {
                    let step = diff * 0.15; // Скорость затухания/появления
                    self.bg_opacity += step;
                }

                // Запускаем выдвижение меню сразу после появления окна (на первом тике)
                if !self.is_closing && !self.panel_revealed {
                    self.panel_revealed = true;
                }

                // Закрываем программу, когда фон исчез и меню уехало
                if self.is_closing && self.bg_opacity <= 0.05 {
                    println!("Closing panel");
                    relm4::main_application().quit();
                }
            }
            Msg::Execute(cmd) => {
                println!("Executing {}", cmd);
                std::process::Command::new("sh").arg("-c").arg(cmd).spawn().ok();
                self.is_closing = true;
                self.panel_revealed = false; // Запускаем скрытие панели
            }
            Msg::Close => { 
                self.is_closing = true; 
                self.panel_revealed = false; // Запускаем скрытие панели
            }
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
        // Обновленный дефолтный CSS для новой архитектуры
        let default_css = "
            window { 
                background-color: transparent; 
            }
            .fullscreen-bg {
                background-color: rgba(0, 0, 0, 0.65); /* Цвет затемнения экрана */
            }
            .panel-box {
                background-color: rgba(30, 30, 46, 1); 
                border-radius: 20px 0 0 20px; 
                border-left: 1px solid #000000;
                border-top: 1px solid #000000;
                border-bottom: 1px solid #000000;
            }
            button { 
                background: #313244; 
                color: #cdd6f4; 
                margin: 5px; 
                padding: 10px 20px; 
                border-radius: 10px; 
                border: none;
            }
            button:focus, button:hover { 
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
    let socket_path = "/tmp/srsq-panel.sock";
    if let Ok(_) = UnixStream::connect(socket_path){
        return;
    }

    unsafe { std::env::set_var("GDK_BACKEND", "wayland"); }
    let app = RelmApp::new("com.hitoshi.srsqpowermenu");
    load_css();
    app.run::<PowerMenu>(());
}
