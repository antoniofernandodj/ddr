use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use clap::{Parser, Subcommand};



#[derive(Debug, Deserialize)]
pub struct CheckHealth {
    pub port: i32,
    pub endpoint: String
}



#[derive(Debug, Deserialize)]
pub struct ContainerConfig {

    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    
    pub environment: Option<Vec<String>>,
    pub command: Option<String>,
    pub check: Option<CheckHealth>
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
    pub check: Option<CheckHealth>
}


#[derive(Debug, Deserialize)]
pub struct InfraConfig {
    pub network_mode: Option<String>,
    pub restart: Option<String>,
    pub env_file: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub environment: Option<Vec<String>>,
    pub _depends_on: Option<Vec<String>>,
    pub mem_limit: Option<String>,
    pub _healthcheck: Option<Value>, // Pode ser mapeado melhor depois
    pub _command: Option<String>,
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
        target: DeployTarget,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum DeployTarget {
    Services,
    Infra,
    Volumes,
    Networks,
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

