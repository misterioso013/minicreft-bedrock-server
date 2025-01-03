use crate::config_manager::ConfigManager;
use crate::server::Server;
use crate::server_admin::ServerAdmin;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct Menu {
    server: Server,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            server: Server::new(),
        }
    }

    pub fn run(&mut self) {
        // Verificar servidores existentes
        match self.server.check_existing_servers() {
            Ok(servers) if !servers.is_empty() => {
                println!("\nServidores Minecraft encontrados em execução:");
                for (i, server) in servers.iter().enumerate() {
                    println!("{}. {}", i + 1, server);
                }
                println!("{}. Iniciar novo servidor", servers.len() + 1);
                println!("{}. Sair", servers.len() + 2);

                print!("\nEscolha uma opção: ");
                io::stdout().flush().unwrap();

                let input = self.get_user_input() as usize;
                if input > 0 && input <= servers.len() {
                    let server_name = &servers[input - 1];
                    match self.server.attach_to_existing(server_name) {
                        Ok(_) => self.run_server_menu(),
                        Err(e) => println!("Erro ao conectar ao servidor: {}", e),
                    }
                } else if input == servers.len() + 1 {
                    // Iniciar novo servidor
                    self.start_new_server();
                } else if input == servers.len() + 2 {
                    println!("Saindo...");
                    return;
                } else {
                    println!("Opção inválida!");
                }
            }
            Ok(_) => {
                // Nenhum servidor encontrado, iniciar novo
                self.start_new_server();
            }
            Err(e) => {
                println!("Erro ao verificar servidores existentes: {}", e);
                self.start_new_server();
            }
        }
    }

    fn start_new_server(&mut self) {
        match self.server.start() {
            Ok(_) => {
                println!("Servidor iniciado com sucesso!");
                self.run_server_menu();
            }
            Err(e) => {
                println!("Não foi possível iniciar o servidor: {}", e);
                self.run_config_menu();
            }
        }
    }

    fn run_server_menu(&mut self) {
        let admin = ServerAdmin::new(PathBuf::from(self.server.get_work_dir()));

        loop {
            self.display_server_options();
            match self.get_user_input() {
                1 => self.admin_menu(&admin),
                2 => {
                    if let Err(e) = self.server.stop() {
                        println!("Erro ao parar servidor: {}", e);
                    } else {
                        println!("Servidor parado com sucesso!");
                        self.run_config_menu();
                        break;
                    }
                }
                3 => {
                    println!("Saindo...");
                    let _ = self.server.stop();
                    break;
                }
                _ => println!("Opção inválida! Por favor, tente novamente."),
            }
        }
    }

    fn admin_menu(&self, admin: &ServerAdmin) {
        loop {
            println!("\n=== Menu de Administração ===");
            println!("1. Listar Jogadores");
            println!("2. Gerenciar Jogador");
            println!("3. Gerenciar Whitelist");
            println!("4. Voltar");

            match self.get_user_input() {
                1 => {
                    if let Err(e) = admin.list_players() {
                        println!("Erro ao listar jogadores: {}", e);
                    }
                }
                2 => self.player_management_menu(admin),
                3 => self.whitelist_menu(admin),
                4 => break,
                _ => println!("Opção inválida!"),
            }
        }
    }

    fn player_management_menu(&self, admin: &ServerAdmin) {
        println!("\nDigite o nome do jogador:");
        let player = self.get_input_string();

        loop {
            println!("\n=== Gerenciar Jogador: {} ===", player);
            println!("1. Mudar Gamemode");
            println!("2. Dar OP");
            println!("3. Remover OP");
            println!("4. Kickar");
            println!("5. Dar Item");
            println!("6. Teleportar");
            println!("7. Voltar");

            match self.get_user_input() {
                1 => {
                    println!("\nEscolha o modo (survival, creative, adventure, spectator):");
                    let mode = self.get_input_string();
                    if let Err(e) = admin.set_gamemode(&player, &mode) {
                        println!("Erro ao mudar gamemode: {}", e);
                    }
                }
                2 => {
                    if let Err(e) = admin.op_player(&player) {
                        println!("Erro ao dar OP: {}", e);
                    }
                }
                3 => {
                    if let Err(e) = admin.deop_player(&player) {
                        println!("Erro ao remover OP: {}", e);
                    }
                }
                4 => {
                    println!("\nDigite o motivo do kick:");
                    let reason = self.get_input_string();
                    if let Err(e) = admin.kick_player(&player, &reason) {
                        println!("Erro ao kickar jogador: {}", e);
                    }
                }
                5 => {
                    println!("\nDigite o item:");
                    let item = self.get_input_string();
                    println!("Digite a quantidade:");
                    let amount: u32 = self.get_input_string().parse().unwrap_or(1);
                    if let Err(e) = admin.give_item(&player, &item, amount) {
                        println!("Erro ao dar item: {}", e);
                    }
                }
                6 => {
                    println!("\nDigite o alvo (jogador ou coordenadas):");
                    let target = self.get_input_string();
                    if let Err(e) = admin.teleport_player(&player, &target) {
                        println!("Erro ao teleportar: {}", e);
                    }
                }
                7 => break,
                _ => println!("Opção inválida!"),
            }
        }
    }

    fn whitelist_menu(&self, admin: &ServerAdmin) {
        loop {
            println!("\n=== Gerenciar Whitelist ===");
            println!("1. Adicionar Jogador");
            println!("2. Remover Jogador");
            println!("3. Voltar");

            match self.get_user_input() {
                1 => {
                    println!("\nDigite o nome do jogador:");
                    let player = self.get_input_string();
                    if let Err(e) = admin.whitelist_add(&player) {
                        println!("Erro ao adicionar à whitelist: {}", e);
                    }
                }
                2 => {
                    println!("\nDigite o nome do jogador:");
                    let player = self.get_input_string();
                    if let Err(e) = admin.whitelist_remove(&player) {
                        println!("Erro ao remover da whitelist: {}", e);
                    }
                }
                3 => break,
                _ => println!("Opção inválida!"),
            }
        }
    }

    fn run_config_menu(&mut self) {
        let config_manager = ConfigManager::new(PathBuf::from(self.server.get_work_dir()));

        loop {
            println!("\n=== Menu de Configuração ===");
            println!("1. Configurar Servidor");
            println!("2. Atualizar Servidor");
            println!("3. Voltar");

            match self.get_user_input() {
                1 => {
                    if let Err(e) = config_manager.configure_server() {
                        println!("Erro ao configurar servidor: {}", e);
                    }
                }
                2 => {
                    if let Err(e) = config_manager.initialize_configs() {
                        println!("Erro ao atualizar servidor: {}", e);
                    }
                }
                3 => {
                    // Tentar iniciar o servidor após configuração
                    match self.server.start() {
                        Ok(_) => {
                            println!("Servidor iniciado com sucesso!");
                            self.run_server_menu();
                        }
                        Err(e) => {
                            println!("Não foi possível iniciar o servidor: {}", e);
                        }
                    }
                    break;
                }
                _ => println!("Opção inválida!"),
            }
        }
    }

    fn display_server_options(&self) {
        println!("\n=== Servidor em Execução ===");
        println!("1. Menu de Administração");
        println!("2. Parar Servidor e Configurar");
        println!("3. Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();
    }

    fn get_user_input(&self) -> u32 {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Falha ao ler entrada");

        input.trim().parse().unwrap_or(0)
    }

    fn get_input_string(&self) -> String {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Falha ao ler entrada");
        input.trim().to_string()
    }
}
