use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct ServerAdmin {
    work_dir: PathBuf,
}

impl ServerAdmin {
    pub fn new(work_dir: PathBuf) -> Self {
        ServerAdmin { work_dir }
    }

    pub fn execute_command(&self, command: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            // Envia o comando
            let status = Command::new("screen")
                .args(["-S", "minecraft", "-X", "stuff", &format!("{}\n", command)])
                .current_dir(&self.work_dir)
                .status()
                .map_err(|e| format!("Erro ao executar comando: {}", e))?;

            if !status.success() {
                return Err("Falha ao executar comando".to_string());
            }

            // Aguarda um momento para o comando ser processado
            thread::sleep(Duration::from_millis(100));

            // Captura a saída do screen
            let output = Command::new("screen")
                .args(["-S", "minecraft", "-X", "hardcopy", "latest.log"])
                .current_dir(&self.work_dir)
                .status()
                .map_err(|e| format!("Erro ao capturar saída: {}", e))?;

            if output.success() {
                // Lê o arquivo de log
                if let Ok(content) = std::fs::read_to_string(self.work_dir.join("latest.log")) {
                    println!("Resposta do servidor:\n{}", content);
                }
            }
        }

        #[cfg(windows)]
        {
            // No Windows, precisamos usar um arquivo de log
            let log_file = self.work_dir.join("server.log");

            // Envia o comando para o servidor
            let mut child = Command::new(&self.work_dir.join("bedrock_server.exe"))
                .arg(command)
                .current_dir(&self.work_dir)
                .spawn()
                .map_err(|e| format!("Erro ao executar comando: {}", e))?;

            // Aguarda um momento para o comando ser processado
            thread::sleep(Duration::from_millis(100));

            // Lê o arquivo de log se existir
            if let Ok(content) = std::fs::read_to_string(&log_file) {
                println!("Resposta do servidor:\n{}", content);
            }
        }

        Ok(())
    }

    pub fn set_gamemode(&self, player: &str, mode: &str) -> Result<(), String> {
        self.execute_command(&format!("/gamemode {} {}", mode, player))
    }

    pub fn op_player(&self, player: &str) -> Result<(), String> {
        self.execute_command(&format!("/op {}", player))
    }

    pub fn deop_player(&self, player: &str) -> Result<(), String> {
        self.execute_command(&format!("/deop {}", player))
    }

    pub fn kick_player(&self, player: &str, reason: &str) -> Result<(), String> {
        self.execute_command(&format!("/kick {} {}", player, reason))
    }

    pub fn whitelist_add(&self, player: &str) -> Result<(), String> {
        self.execute_command(&format!("/whitelist add {}", player))
    }

    pub fn whitelist_remove(&self, player: &str) -> Result<(), String> {
        self.execute_command(&format!("/whitelist remove {}", player))
    }

    pub fn list_players(&self) -> Result<(), String> {
        self.execute_command("/list")
    }

    pub fn teleport_player(&self, player: &str, target: &str) -> Result<(), String> {
        self.execute_command(&format!("/tp {} {}", player, target))
    }

    pub fn give_item(&self, player: &str, item: &str, amount: u32) -> Result<(), String> {
        self.execute_command(&format!("/give {} {} {}", player, item, amount))
    }
}
