use rand::Rng;
use reqwest;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio;

pub struct Config {
    work_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let work_dir = PathBuf::from("server");
        Config { work_dir }
    }

    pub fn run(&self) {
        loop {
            self.display_options();
            match self.get_user_input() {
                1 => self.configurar_servidor(),
                2 => self.atualizar_servidor(),
                3 => {
                    println!("Saindo...");
                    break;
                }
                _ => println!("Opção inválida! Por favor, tente novamente."),
            }
        }
    }

    fn display_options(&self) {
        println!("\n=== Configurações do Servidor ===");
        println!("1. Configurar servidor");
        println!("2. Atualizar servidor");
        println!("3. Voltar");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();
    }

    fn get_user_input(&self) -> u32 {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Erro ao ler entrada");
        input.trim().parse().unwrap_or(0)
    }

    fn configurar_servidor(&self) {
        println!("Configurando servidor...");

        // Criar diretórios necessários
        let dirs = [
            &self.work_dir,
            &self.work_dir.join("downloads"),
            &self.work_dir.join("backups"),
            &self.work_dir.join("logs"),
            &self.work_dir.join("worlds"),
        ];

        for dir in dirs.iter() {
            if !dir.exists() {
                fs::create_dir_all(dir).expect(&format!("Erro ao criar diretório {:?}", dir));
                println!("Diretório {:?} criado com sucesso", dir);
            }
        }

        // Iniciar download assíncrono do servidor
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.download_server());
    }

    async fn download_server(&self) {
        let rand_num: u32 = rand::thread_rng().gen_range(1..5000);
        let user_agent = format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
            (KHTML, like Gecko) Chrome/90.0.{}.212 Safari/537.36",
            rand_num
        );

        println!("Verificando última versão do servidor Minecraft Bedrock...");

        // Determina o sistema operacional
        let os_type = env::consts::OS;
        println!("Sistema operacional detectado: {}", os_type);

        let client = reqwest::Client::new();
        let response = client
            .get("https://www.minecraft.net/pt-br/download/server/bedrock")
            .header("Accept-Encoding", "identity")
            .header("Accept-Language", "en")
            .header("User-Agent", &user_agent)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let body = resp.text().await.unwrap();
                if let Some(download_url) = self.extract_download_url(&body, os_type) {
                    println!("URL de download encontrada: {}", download_url);
                    self.download_server_files(&download_url).await;
                } else {
                    println!("Não foi possível encontrar a URL de download do servidor");
                }
            }
            Err(e) => println!("Erro ao verificar atualizações: {}", e),
        }
    }

    fn extract_download_url(&self, html: &str, os_type: &str) -> Option<String> {
        let search_string = match os_type {
            "windows" => "https://www.minecraft.net/bedrockdedicatedserver/bin-win/",
            "linux" => "https://www.minecraft.net/bedrockdedicatedserver/bin-linux/",
            _ => {
                println!("Sistema operacional não suportado: {}", os_type);
                return None;
            }
        };

        if let Some(start) = html.find(search_string) {
            if let Some(end) = html[start..].find("\"") {
                return Some(html[start..start + end].to_string());
            }
        }

        // Fallback para URLs fixas caso não encontre no HTML
        match os_type {
            "windows" => Some(
                "https://www.minecraft.net/bedrockdedicatedserver/bin-win/bedrock-server-1.21.51.02.zip".to_string(),
            ),
            "linux" => Some(
                "https://www.minecraft.net/bedrockdedicatedserver/bin-linux/bedrock-server-1.21.51.02.zip".to_string(),
            ),
            _ => None,
        }
    }

    async fn download_server_files(&self, url: &str) {
        println!("Baixando servidor Minecraft Bedrock...");
        println!("URL de download: {}", url);

        let response = reqwest::get(url).await;
        match response {
            Ok(resp) => {
                let bytes = resp.bytes().await.unwrap();
                let filename = url.split('/').last().unwrap_or("server.zip");
                let download_path = self.work_dir.join("downloads").join(filename);

                fs::write(&download_path, bytes).expect("Erro ao salvar arquivo");
                println!("Download concluído! Arquivo salvo em: {:?}", download_path);
                println!("Extraindo arquivos...");

                self.extract_server_files(download_path.to_str().unwrap());
            }
            Err(e) => println!("Erro ao baixar servidor: {}", e),
        }
    }

    fn extract_server_files(&self, zip_path: &str) {
        let file = fs::File::open(zip_path).expect("Erro ao abrir arquivo ZIP");
        let mut archive = zip::ZipArchive::new(file).expect("Erro ao ler arquivo ZIP");

        println!("Extraindo {} arquivos...", archive.len());

        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let outpath = match file.enclosed_name() {
                    Some(path) => self.work_dir.join(path),
                    None => continue,
                };

                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath).unwrap();
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p).unwrap();
                        }
                    }
                    let mut outfile = fs::File::create(&outpath).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }

                // Definir permissões no Linux
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                    }
                }
            }
        }

        // Após a extração, define permissões do executável no Linux
        #[cfg(unix)]
        {
            let executable = self.work_dir.join("bedrock_server");
            if executable.exists() {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&executable)
                    .expect("Erro ao ler permissões")
                    .permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&executable, perms)
                    .expect("Erro ao definir permissões do executável");
                println!("Permissões do executável configuradas");
            }
        }

        println!("Extração concluída em {:?}!", self.work_dir);
    }

    fn atualizar_servidor(&self) {
        println!("Verificando atualizações...");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.download_server());
    }
}
