#[allow(dead_code, unused)]

mod networks;
mod services;
mod utils;
mod volumes;
mod models;

use serde_yaml::Value;
use clap::Parser;

use crate::models::{Cli, Commands};
use crate::networks::handle_networks;
use crate::services::handle_group;
use crate::utils::{process_deployment_file};
use crate::volumes::handle_volumes;


fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ssh_config = utils::get_ssh_config()?;


    match cli.command {

        Commands::Deploy { group_name, dry_run, config} => {

            let deploy_yaml = process_deployment_file(&config)?;
            let deploy_map = deploy_yaml.as_mapping().unwrap();

            if let Some(_) = deploy_map.get(Value::String(group_name.clone())) {

                if ["define"].contains(&group_name.as_str()) {
                    return Ok(())
                }

                else if ["networks"].contains(&group_name.as_str()) {
                    handle_networks(
                        &ssh_config,
                        deploy_map,
                        dry_run
                    )?;

                    return Ok(())
                }

                else if ["volumes"].contains(&group_name.as_str()) {
                    handle_volumes(
                        &ssh_config,
                        deploy_map,
                        dry_run)?;

                    return Ok(())
                }


                else {
                    handle_group(
                        &ssh_config,
                        &group_name,
                        deploy_map,
                        dry_run
                    )?;

                    return Ok(())
                }

            } else {
                println!("Grupo não encontrado!");
            }
                
        }
        
    }
    
    Ok(())
}

