use serde::Deserialize;
use serde_yaml::{Mapping};
use clap::{Parser, Subcommand};





#[derive(Debug, Deserialize, Clone)]
pub struct RemoteHealthCheck {
    pub port: Option<i32>,
    pub endpoint: Option<String>
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
#[command(name = "ddr")]
#[command(about = "CLI para gerar comandos Docker a partir do deploy.yaml", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Deploy {
        #[arg(value_enum)]
        #[clap(long)]
        group_name: String,
        /// Simula a execução sem aplicar mudanças
        #[clap(long)]
        dry_run: bool,
    },
}


pub struct SSHConfig {
    pub user: String,
    pub host: String,
    pub password: String,
    pub from_dir: String
}

impl SSHConfig {
    pub fn new(user: String, host: String, password: String, from_dir: String) -> Self {
        SSHConfig {
            user: user,
            host: host,
            password: password,
            from_dir: from_dir
        }
    }
}

