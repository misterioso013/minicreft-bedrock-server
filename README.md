# Minecraft Bedrock Server Manager

Um gerenciador de servidor Minecraft Bedrock escrito em Rust que permite baixar, configurar e administrar servidores facilmente.

(Ainda em desenvolvimento)

## Características

- 🚀 Download e instalação automática do servidor
- ⚙️ Configuração interativa do servidor
- 👥 Gerenciamento de jogadores (whitelist, ops, etc)
- 🎮 Comandos administrativos em tempo real
- 🔄 Suporte a múltiplos servidores
- 💻 Compatível com Windows e Linux

## Requisitos

### Linux
- Ubuntu 18.04 ou superior
- Screen (`sudo apt-get install screen`)
- Rust e Cargo

### Windows
- Windows 10 versão 1703 ou superior
- Windows Server 2016 ou superior
- Rust e Cargo

## Instalação

1. Clone o repositório:
```bash
git clone https://github.com/misterioso013/minecraft-bedrock-server
cd minecraft-bedrock-server
```

2. Compile o projeto:
```bash
cargo build --release
```

3. Execute o gerenciador:
```bash
cargo run
```

## Uso

### Primeira Execução

1. O programa verificará servidores existentes
2. Se nenhum servidor for encontrado, iniciará o processo de configuração
3. Baixará automaticamente a última versão do Minecraft Bedrock Server
4. Criará os arquivos de configuração necessários

### Menu Principal

- **Administrar Servidor Existente**: Conecta a um servidor em execução
- **Iniciar Novo Servidor**: Configura e inicia uma nova instância
- **Sair**: Encerra o programa

### Menu de Administração

- Listar jogadores conectados
- Gerenciar jogadores:
  - Mudar gamemode
  - Dar/remover OP
  - Kickar jogadores
  - Dar itens
  - Teleportar
- Gerenciar whitelist:
  - Adicionar jogadores
  - Remover jogadores

### Configurações

O servidor pode ser configurado através do menu ou editando diretamente os arquivos:

- `server.properties`: Configurações gerais do servidor
- `permissions.json`: Permissões dos jogadores
- `allowlist.json`: Lista de jogadores permitidos

## Estrutura de Diretórios

```
minecraft-bedrock-server/
├── server/              # Arquivos do servidor
│   ├── worlds/         # Mundos do servidor
│   ├── logs/           # Logs do servidor
│   └── ...
└── src/                # Código-fonte
```

## Comandos Disponíveis

| Comando | Descrição |
|---------|-----------|
| `/list` | Lista jogadores conectados |
| `/gamemode <modo> <jogador>` | Altera o modo de jogo |
| `/tp <jogador> <destino>` | Teleporta jogador |
| `/give <jogador> <item> <quantidade>` | Dá itens ao jogador |
| `/kick <jogador> <motivo>` | Expulsa jogador do servidor |
| `/op <jogador>` | Torna jogador operador |
| `/deop <jogador>` | Remove status de operador |

## Contribuindo

Contribuições são bem-vindas! Por favor, sinta-se à vontade para enviar pull requests.

1. Fork o projeto
2. Crie sua branch de feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## Licença

Este projeto está licenciado sob a [MIT License](LICENSE).

## Agradecimentos

- Mojang Studios pelo Minecraft
- Comunidade Rust
- Contribuidores do projeto

## Suporte

Se você encontrar algum problema ou tiver sugestões, por favor abra uma issue no GitHub.