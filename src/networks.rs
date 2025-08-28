
use std::fs;
use std::io::Read;

use serde::Deserialize;
use serde_yaml::{from_value, Mapping, Value};
use clap::{Parser, Subcommand};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use dotenvy::from_filename;
use std::env;

use crate::models::SSHConfig;
use crate::utils::{docker_run, get_session};


pub fn handle_networks(
    ssh_config: &SSHConfig,
    deploy_map: &Mapping,
    dry_run: bool
) -> anyhow::Result<()> {
    let group_config: Mapping = deploy_map
        .get(&Value::String("networks".to_string()))
        .cloned()
        .expect("Group config 'volumes' não encontrado!")
        .as_mapping()
        .expect("'volumes' não é um mapping")
        .to_owned(); // agora é Mapping, iterável

    let session = get_session(ssh_config)?;
    for (network_name, network_value) in group_config {
        let mut network_name: String = from_value(network_name).unwrap();

        let mut cmd = format!("docker network create {}", network_name);

        if let Some(network_map) = network_value.as_mapping() {
            if let Some(name) = network_map.get(
                &Value::String("name".to_string())
            ) {
                if let Some(name) = name.as_str() {
                    cmd = format!("docker network create {}", name);
                }
            }

            // driver
            if let Some(driver) = network_map.get(
                &Value::String("driver".to_string())
            ) {
                if let Some(driver_str) = driver.as_str() {
                    cmd += &format!(" --driver {}", driver_str);
                }
            }
            // subnet
            if let Some(ipam) = network_map.get(
                &Value::String("ipam".to_string())
            ) {
                if let Some(ipam_map) = ipam.as_mapping() {
                    if let Some(configs) = ipam_map.get(
                        &Value::String("config".to_string())
                    ) {
                        if let Some(configs_seq) = configs.as_sequence() {
                            for config in configs_seq {
                                if let Some(config_map) = config.as_mapping() {
                                    if let Some(subnet) = config_map.get(
                                        &Value::String("subnet".to_string())
                                    ) {
                                        if let Some(subnet_str) = subnet.as_str() {
                                            cmd += &format!(" --subnet {}", subnet_str);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("\n{}", cmd);

        if !dry_run {
            docker_run(&session, cmd)?;
        }
    }

    Ok(())
}

