mod config;
mod config_manager;
mod menu;
mod server;
mod server_admin;

fn main() {
    println!("Minecraft Bedrock Server");
    let mut menu = menu::Menu::new();
    menu.run();
}
