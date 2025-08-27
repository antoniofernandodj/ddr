#[allow(dead_code, unused)]

mod networks;
mod services;
mod utils;
mod volumes;
mod infra;
mod models;

use serde_yaml::Value;
use clap::Parser;

use crate::models::{Cli, Commands, DeployTarget};
use crate::services::handle_services;
use crate::utils::read_deployment_file;
use crate::infra::handle_infra;
use crate::volumes::handle_volumes;
use crate::networks::handle_networks;


fn main() -> anyhow::Result<()> {
    println!("Iniciando");
    let cli = Cli::parse();

    let ssh_config = utils::get_ssh_config()?;

    // Lê o deploy.yaml
    let deploy_yaml = read_deployment_file("deploy.yaml")?;
    let deploy_map = deploy_yaml.as_mapping().unwrap();

    match cli.command {
        Commands::Deploy { target } => match target {
            DeployTarget::Services => {
                if let Some(group) = deploy_map.get(
                    &Value::String("services".to_string())
                ) {
                    handle_services(&ssh_config, group.clone())?;
                }
            }
            DeployTarget::Infra => {
                if let Some(group) = deploy_map.get(
                    &Value::String("infra".to_string())
                ) {
                    handle_infra(&ssh_config, group.clone())?;
                }
            }
            DeployTarget::Volumes => {
                if let Some(group) = deploy_map.get(
                    &Value::String("volumes".to_string())
                ) {
                    handle_volumes(&ssh_config, group.clone())?;
                }
            }
            DeployTarget::Networks => {
                if let Some(group) = deploy_map.get(
                    &Value::String("networks".to_string())) {
                    handle_networks(&ssh_config, group.clone());
                }
            }
        },
    }

    Ok(())
}

