use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_yaml::Mapping;

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteHealthCheck {
    pub port: Option<i32>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheck {
    pub test: Vec<String>,
    pub interval: String,
    pub timeout: String,
    pub retries: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerConfig {
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,

    pub environment: Option<Vec<String>>,
    pub command: Option<String>,
    pub remotecheck: Option<RemoteHealthCheck>,
    pub healthcheck: Option<HealthCheck>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub image: Option<String>,
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub environment: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub instances: Mapping,
    pub remotecheck: Option<RemoteHealthCheck>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct InfraConfig {
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub environment: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub mem_limit: Option<String>,
    pub healthcheck: Option<HealthCheck>,
    pub command: Option<String>,
    pub instances: Mapping, // Algumas infra pode ter instâncias
}

#[derive(Parser)]
#[command(
    name = "ddr",
    about = "CLI para gerar comandos Docker a partir de um arquivo de \
            configuração (default: deploy.yaml)",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Simula a execução sem aplicar mudanças
    #[arg(short, long)]
    pub dry_run: bool,
    /// Define o arquivo de configuração a ser usado
    #[arg(short, long, default_value = "deploy.yaml")]
    pub config: String,
    /// Define uma lista de variáveis de ambiente a ser usadas
    #[arg(short, long)]
    pub envs: Option<Vec<String>>,
    /// Define o arquivo de configuração de variáveis de ambiente a ser usado
    #[arg(short, long, default_value = "infra.secrets.env")]
    pub env_config: String,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Executa o processo de deploy para um grupo",
        long_about = "Este subcomando permite rodar o processo de deploy \
                      para um grupo específico definido no arquivo de configuração. \
                      Pode simular a execução sem aplicar mudanças (dry-run)."
    )]
    Deploy {
        /// Seleciona o grupo a ser processado
        #[arg(short, long)]
        group_name: String,
    },
}

pub struct SSHConfig {
    pub user: String,
    pub host: String,
    pub password: String,
    pub from_dir: String,
}

impl SSHConfig {
    pub fn new(user: String, host: String, password: String, from_dir: String) -> Self {
        SSHConfig {
            user: user,
            host: host,
            password: password,
            from_dir: from_dir,
        }
    }
}
