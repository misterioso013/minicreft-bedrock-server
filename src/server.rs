use crate::config_manager::ConfigManager;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct Server {
    process: Option<Child>,
    is_running: Arc<AtomicBool>,
    work_dir: String,
}

impl Server {
    pub fn new() -> Self {
        // Obter o caminho absoluto para o diretório do servidor
        let current_dir = env::current_dir().expect("Erro ao obter diretório atual");
        let work_dir = current_dir.join("server").to_string_lossy().to_string();

        Server {
            process: None,
            is_running: Arc::new(AtomicBool::new(false)),
            work_dir,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    fn check_screen_installed(&self) -> bool {
        Command::new("which")
            .arg("screen")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            return Err("Servidor já está em execução".to_string());
        }

        #[cfg(windows)]
        return self.start_windows();

        #[cfg(unix)]
        return self.start_unix();
    }

    #[cfg(windows)]
    fn start_windows(&mut self) -> Result<(), String> {
        let executable = "bedrock_server.exe";
        let server_path = Path::new(&self.work_dir).join(executable);
        println!("Tentando iniciar servidor em: {:?}", server_path);

        if !server_path.exists() {
            return Err(format!(
                "Servidor não encontrado em {:?}. Execute a configuração primeiro.",
                server_path
            ));
        }

        // Configurações do Windows para executar em segundo plano
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let mut command = Command::new(&server_path);
        command
            .current_dir(&self.work_dir)
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);

        match command.spawn() {
            Ok(child) => {
                self.process = Some(child);
                self.is_running.store(true, Ordering::Relaxed);
                println!("Servidor iniciado com sucesso!");
                Ok(())
            }
            Err(e) => Err(format!("Erro ao iniciar servidor: {}", e)),
        }
    }

    #[cfg(unix)]
    fn start_unix(&mut self) -> Result<(), String> {
        // Verificar se o screen está instalado
        if !self.check_screen_installed() {
            return Err("O programa 'screen' não está instalado. \
                Por favor, instale usando:\n\
                Ubuntu/Debian: sudo apt-get install screen\n\
                Fedora: sudo dnf install screen\n\
                Arch Linux: sudo pacman -S screen"
                .to_string());
        }

        // Inicializar configurações
        let config_manager = ConfigManager::new(PathBuf::from(&self.work_dir));
        config_manager.initialize_configs()?;

        let executable = if cfg!(windows) {
            "bedrock_server.exe"
        } else {
            "bedrock_server" // Removido ./ pois usaremos caminho absoluto
        };

        let server_path = Path::new(&self.work_dir).join(executable);
        println!("Tentando iniciar servidor em: {:?}", server_path);

        if !server_path.exists() {
            return Err(format!(
                "Servidor não encontrado em {:?}. Execute a configuração primeiro.",
                server_path
            ));
        }

        // Define permissões de execução no Linux
        #[cfg(unix)]
        {
            let metadata =
                fs::metadata(&server_path).map_err(|e| format!("Erro ao ler permissões: {}", e))?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755); // rwxr-xr-x
            fs::set_permissions(&server_path, perms)
                .map_err(|e| format!("Erro ao definir permissões: {}", e))?;
        }

        // Configurar variáveis de ambiente necessárias
        let mut command = Command::new(&server_path);
        command.current_dir(&self.work_dir);

        // Configurar LD_LIBRARY_PATH no Linux
        #[cfg(unix)]
        {
            // Usar o work_dir diretamente como LD_LIBRARY_PATH
            command.env("LD_LIBRARY_PATH", &self.work_dir);
        }

        // Criar diretório de logs se não existir
        let logs_dir = Path::new(&self.work_dir).join("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(&logs_dir)
                .map_err(|e| format!("Erro ao criar diretório de logs: {}", e))?;
        }

        // Verificar se os arquivos necessários existem
        let files_to_check = ["allowlist.json", "permissions.json", "server.properties"];
        for file in files_to_check.iter() {
            let file_path = Path::new(&self.work_dir).join(file);
            if !file_path.exists() {
                let content = match *file {
                    "server.properties" => "server-name=Dedicated Server\nserver-port=19132\ngamemode=survival\ndifficulty=normal\nallow-cheats=false\nmax-players=10\nonline-mode=true\nwhite-list=false\nview-distance=32\ntick-distance=4\nplayer-idle-timeout=30\nmax-threads=8\ndefault-player-permission-level=member\ntexturepack-required=false\ncontent-log-file-enabled=false\ncompression-threshold=1\nserver-authoritative-movement=server-auth\nplayer-movement-score-threshold=20\nplayer-movement-action-direction-threshold=0.85\nplayer-movement-distance-threshold=0.3\nplayer-movement-duration-threshold-in-ms=500\ncorrect-player-movement=false\nserver-authoritative-block-breaking=false\n",
                    _ => "[]"
                };
                fs::write(&file_path, content)
                    .map_err(|e| format!("Erro ao criar {}: {}", file, e))?;
                println!("Arquivo {} criado com configuração padrão", file);
            }
        }

        // Iniciar o servidor em uma sessão screen
        let screen_status = Command::new("screen")
            .args(["-dmS", "minecraft"])
            .current_dir(&self.work_dir)
            .status()
            .map_err(|e| format!("Erro ao criar sessão screen: {}", e))?;

        if !screen_status.success() {
            return Err("Falha ao criar sessão screen".to_string());
        }

        // Pequena pausa para garantir que a sessão screen foi criada
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Iniciar o servidor dentro da sessão screen
        let status = Command::new("screen")
            .args([
                "-S",
                "minecraft",
                "-X",
                "stuff",
                &format!("{}\n", server_path.display()),
            ])
            .current_dir(&self.work_dir)
            .status()
            .map_err(|e| format!("Erro ao iniciar servidor: {}", e))?;

        if status.success() {
            println!("Servidor iniciado com sucesso!");
            self.is_running.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err("Falha ao iniciar servidor".to_string())
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        #[cfg(windows)]
        {
            if let Some(mut child) = self.process.take() {
                match child.kill() {
                    Ok(_) => {
                        self.is_running.store(false, Ordering::Relaxed);
                        Ok(())
                    }
                    Err(e) => Err(format!("Erro ao parar servidor: {}", e)),
                }
            } else {
                Err("Servidor não está em execução".to_string())
            }
        }

        #[cfg(unix)]
        {
            // Enviar comando de stop via screen
            let status = Command::new("screen")
                .args(["-S", "minecraft", "-X", "stuff", "stop\n"])
                .current_dir(&self.work_dir)
                .status()
                .map_err(|e| format!("Erro ao parar servidor: {}", e))?;

            if status.success() {
                self.is_running.store(false, Ordering::Relaxed);
                Ok(())
            } else {
                Err("Falha ao parar servidor".to_string())
            }
        }
    }

    pub fn get_work_dir(&self) -> &str {
        &self.work_dir
    }

    pub fn check_existing_servers(&self) -> Result<Vec<String>, String> {
        #[cfg(unix)]
        {
            let output = Command::new("screen")
                .args(["-ls"])
                .output()
                .map_err(|e| format!("Erro ao verificar servidores: {}", e))?;

            let screen_list = String::from_utf8_lossy(&output.stdout);
            let mut servers = Vec::new();

            for line in screen_list.lines() {
                if line.contains("minecraft") {
                    if let Some(name) = line.split('.').nth(1) {
                        servers.push(name.trim().to_string());
                    }
                }
            }

            Ok(servers)
        }

        #[cfg(windows)]
        {
            // No Windows, verificamos se existe um processo do servidor rodando
            let output = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq bedrock_server.exe"])
                .output()
                .map_err(|e| format!("Erro ao verificar servidores: {}", e))?;

            let process_list = String::from_utf8_lossy(&output.stdout);
            let mut servers = Vec::new();

            if process_list.contains("bedrock_server.exe") {
                servers.push("bedrock_server".to_string());
            }

            Ok(servers)
        }
    }

    pub fn attach_to_existing(&mut self, server_name: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            self.is_running.store(true, Ordering::Relaxed);
            println!("Conectado ao servidor existente: {}", server_name);
            Ok(())
        }

        #[cfg(windows)]
        {
            self.is_running.store(true, Ordering::Relaxed);
            println!("Conectado ao servidor existente");
            Ok(())
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}
