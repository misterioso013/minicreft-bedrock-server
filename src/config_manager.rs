use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct ServerProperties {
    #[serde(rename = "server-name")]
    server_name: String,
    #[serde(rename = "server-port")]
    server_port: u16,
    gamemode: String,
    difficulty: String,
    #[serde(rename = "allow-cheats")]
    allow_cheats: bool,
    #[serde(rename = "max-players")]
    max_players: u32,
    #[serde(rename = "online-mode")]
    online_mode: bool,
    #[serde(rename = "white-list")]
    white_list: bool,
    #[serde(rename = "view-distance")]
    view_distance: u32,
    #[serde(rename = "tick-distance")]
    tick_distance: u32,
    #[serde(rename = "player-idle-timeout")]
    player_idle_timeout: u32,
    #[serde(rename = "max-threads")]
    max_threads: u32,
    #[serde(rename = "default-player-permission-level")]
    default_player_permission_level: String,
    #[serde(rename = "texturepack-required")]
    texturepack_required: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Permission {
    permission: String,
    xuid: String,
}

#[derive(Serialize, Deserialize)]
pub struct AllowlistEntry {
    name: String,
    xuid: String,
    #[serde(rename = "ignoresPlayerLimit")]
    ignores_player_limit: bool,
}

pub struct ConfigManager {
    work_dir: PathBuf,
}

impl ConfigManager {
    pub fn new(work_dir: PathBuf) -> Self {
        ConfigManager { work_dir }
    }

    pub fn initialize_configs(&self) -> Result<(), String> {
        self.create_default_server_properties()?;
        self.create_default_permissions()?;
        self.create_default_allowlist()?;
        Ok(())
    }

    fn create_default_server_properties(&self) -> Result<(), String> {
        let properties = ServerProperties {
            server_name: "Dedicated Server".to_string(),
            server_port: 19132,
            gamemode: "survival".to_string(),
            difficulty: "normal".to_string(),
            allow_cheats: false,
            max_players: 10,
            online_mode: true,
            white_list: false,
            view_distance: 32,
            tick_distance: 4,
            player_idle_timeout: 30,
            max_threads: 8,
            default_player_permission_level: "member".to_string(),
            texturepack_required: false,
        };

        let content = self.properties_to_string(&properties)?;
        let path = self.work_dir.join("server.properties");
        fs::write(&path, content).map_err(|e| format!("Erro ao criar server.properties: {}", e))?;
        println!("Arquivo server.properties criado com configurações padrão");
        Ok(())
    }

    fn properties_to_string(&self, props: &ServerProperties) -> Result<String, String> {
        let mut content = String::new();
        content.push_str(&format!("server-name={}\n", props.server_name));
        content.push_str(&format!("server-port={}\n", props.server_port));
        content.push_str(&format!("gamemode={}\n", props.gamemode));
        content.push_str(&format!("difficulty={}\n", props.difficulty));
        content.push_str(&format!("allow-cheats={}\n", props.allow_cheats));
        content.push_str(&format!("max-players={}\n", props.max_players));
        content.push_str(&format!("online-mode={}\n", props.online_mode));
        content.push_str(&format!("white-list={}\n", props.white_list));
        content.push_str(&format!("view-distance={}\n", props.view_distance));
        content.push_str(&format!("tick-distance={}\n", props.tick_distance));
        content.push_str(&format!(
            "player-idle-timeout={}\n",
            props.player_idle_timeout
        ));
        content.push_str(&format!("max-threads={}\n", props.max_threads));
        content.push_str(&format!(
            "default-player-permission-level={}\n",
            props.default_player_permission_level
        ));
        content.push_str(&format!(
            "texturepack-required={}\n",
            props.texturepack_required
        ));
        Ok(content)
    }

    fn create_default_permissions(&self) -> Result<(), String> {
        let permissions: Vec<Permission> = vec![];
        let content = serde_json::to_string_pretty(&permissions)
            .map_err(|e| format!("Erro ao serializar permissions.json: {}", e))?;

        // Criar permissions.json na raiz
        let path = self.work_dir.join("permissions.json");
        fs::write(&path, &content).map_err(|e| format!("Erro ao criar permissions.json: {}", e))?;

        // Criar diretório config/default se não existir
        let default_config_dir = self.work_dir.join("config").join("default");
        fs::create_dir_all(&default_config_dir)
            .map_err(|e| format!("Erro ao criar diretório config/default: {}", e))?;

        // Criar permissions.json no diretório config/default
        let default_path = default_config_dir.join("permissions.json");
        fs::write(&default_path, content)
            .map_err(|e| format!("Erro ao criar config/default/permissions.json: {}", e))?;

        println!("Arquivos de permissões criados com configurações padrão");
        Ok(())
    }

    fn create_default_allowlist(&self) -> Result<(), String> {
        let allowlist: Vec<AllowlistEntry> = vec![];
        let content = serde_json::to_string_pretty(&allowlist)
            .map_err(|e| format!("Erro ao serializar allowlist.json: {}", e))?;

        let path = self.work_dir.join("allowlist.json");
        fs::write(&path, content).map_err(|e| format!("Erro ao criar allowlist.json: {}", e))?;

        println!("Arquivo allowlist.json criado com configuração padrão");
        Ok(())
    }

    pub fn configure_server(&self) -> Result<(), String> {
        let properties = self.read_server_properties()?;
        let updated_properties = self.interactive_config(properties)?;
        self.save_server_properties(&updated_properties)
    }

    fn read_server_properties(&self) -> Result<ServerProperties, String> {
        let path = self.work_dir.join("server.properties");
        let file =
            fs::File::open(&path).map_err(|e| format!("Erro ao abrir server.properties: {}", e))?;
        let reader = BufReader::new(file);
        let mut props = ServerProperties {
            server_name: "Dedicated Server".to_string(),
            server_port: 19132,
            gamemode: "survival".to_string(),
            difficulty: "normal".to_string(),
            allow_cheats: false,
            max_players: 10,
            online_mode: true,
            white_list: false,
            view_distance: 32,
            tick_distance: 4,
            player_idle_timeout: 30,
            max_threads: 8,
            default_player_permission_level: "member".to_string(),
            texturepack_required: false,
        };

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Erro ao ler linha: {}", e))?;
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "server-name" => props.server_name = value.to_string(),
                    "server-port" => props.server_port = value.parse().unwrap_or(19132),
                    "gamemode" => props.gamemode = value.to_string(),
                    "difficulty" => props.difficulty = value.to_string(),
                    "allow-cheats" => props.allow_cheats = value.parse().unwrap_or(false),
                    "max-players" => props.max_players = value.parse().unwrap_or(10),
                    "online-mode" => props.online_mode = value.parse().unwrap_or(true),
                    "white-list" => props.white_list = value.parse().unwrap_or(false),
                    "view-distance" => props.view_distance = value.parse().unwrap_or(32),
                    "tick-distance" => props.tick_distance = value.parse().unwrap_or(4),
                    "player-idle-timeout" => {
                        props.player_idle_timeout = value.parse().unwrap_or(30)
                    }
                    "max-threads" => props.max_threads = value.parse().unwrap_or(8),
                    "default-player-permission-level" => {
                        props.default_player_permission_level = value.to_string()
                    }
                    "texturepack-required" => {
                        props.texturepack_required = value.parse().unwrap_or(false)
                    }
                    _ => {}
                }
            }
        }

        Ok(props)
    }

    fn interactive_config(&self, mut props: ServerProperties) -> Result<ServerProperties, String> {
        println!("\n=== Configuração do Servidor ===\n");

        props.server_name = self.prompt_string(
            "Nome do Servidor",
            &props.server_name,
            "Digite o nome do servidor",
        )?;

        props.server_port = self.prompt_number(
            "Porta do Servidor",
            props.server_port,
            "Digite a porta do servidor (1-65535)",
        )?;

        props.gamemode = self.prompt_options(
            "Modo de Jogo",
            &props.gamemode,
            &["survival", "creative", "adventure"],
            "Escolha o modo de jogo",
        )?;

        props.difficulty = self.prompt_options(
            "Dificuldade",
            &props.difficulty,
            &["peaceful", "easy", "normal", "hard"],
            "Escolha a dificuldade",
        )?;

        props.allow_cheats = self.prompt_bool("Permitir Cheats", props.allow_cheats)?;
        props.max_players = self.prompt_number(
            "Máximo de Jogadores",
            props.max_players,
            "Digite o número máximo de jogadores",
        )?;
        props.online_mode = self.prompt_bool("Modo Online", props.online_mode)?;
        props.white_list = self.prompt_bool("Usar Whitelist", props.white_list)?;

        Ok(props)
    }

    fn prompt_string(&self, label: &str, current: &str, help: &str) -> Result<String, String> {
        println!("{} [{}]", label, current);
        println!("({})", help);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Erro ao ler entrada: {}", e))?;
        let input = input.trim();
        Ok(if input.is_empty() {
            current.to_string()
        } else {
            input.to_string()
        })
    }

    fn prompt_number<T>(&self, label: &str, current: T, help: &str) -> Result<T, String>
    where
        T: std::str::FromStr + std::fmt::Display,
    {
        println!("{} [{}]", label, current);
        println!("({})", help);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Erro ao ler entrada: {}", e))?;
        let input = input.trim();
        Ok(if input.is_empty() {
            current
        } else {
            input
                .parse()
                .map_err(|_| format!("Valor inválido para {}", label))?
        })
    }

    fn prompt_bool(&self, label: &str, current: bool) -> Result<bool, String> {
        println!(
            "{} [{}/{}]",
            label,
            if current { "S" } else { "N" },
            if current { "n" } else { "s" }
        );
        println!("(Digite S para Sim ou N para Não)");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Erro ao ler entrada: {}", e))?;
        let input = input.trim().to_lowercase();
        Ok(if input.is_empty() {
            current
        } else {
            match input.as_str() {
                "s" | "sim" | "y" | "yes" => true,
                "n" | "não" | "nao" | "no" => false,
                _ => current,
            }
        })
    }

    fn prompt_options(
        &self,
        label: &str,
        current: &str,
        options: &[&str],
        help: &str,
    ) -> Result<String, String> {
        println!("{} [{}]", label, current);
        println!("({})", help);
        println!("Opções disponíveis: {}", options.join(", "));
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Erro ao ler entrada: {}", e))?;
        let input = input.trim();
        Ok(if input.is_empty() {
            current.to_string()
        } else if options.contains(&input) {
            input.to_string()
        } else {
            println!("Opção inválida, mantendo valor atual");
            current.to_string()
        })
    }

    fn save_server_properties(&self, props: &ServerProperties) -> Result<(), String> {
        let content = self.properties_to_string(props)?;
        let path = self.work_dir.join("server.properties");
        fs::write(&path, content)
            .map_err(|e| format!("Erro ao salvar server.properties: {}", e))?;
        println!("Configurações salvas com sucesso!");
        Ok(())
    }
}
