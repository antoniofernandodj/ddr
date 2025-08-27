use std::{thread, time::Duration};

use serde_yaml::{from_value, Value};
use ssh2::Session;
use reqwest::blocking::Client;
use crate::{
    models::{
        ContainerConfig,
        SSHConfig,
        ServiceConfig
    },
    utils::{
        docker_load_and_run,
        docker_save,
        get_session,
        scp_send
    }
};



pub fn handle_services(
    ssh_config: &SSHConfig,
    group_config: Value
) -> anyhow::Result<()> {

    let session: Session = get_session(ssh_config)?;
    let group_config: serde_yaml::Mapping = group_config
        .as_mapping()
        .unwrap()
        .to_owned();

    for (image_name, service_config) in group_config {
        let image_name: String = from_value(image_name)?;
        let tar_file: String = format!("{}.tar", image_name.replace("/", "_").replace(":", "_"));

        dbg!("Salvando imagem em tar file");
        docker_save(&image_name, &tar_file)?;
        dbg!("Salvou a imagem em uma tar file");

        let service_config: ServiceConfig = from_value(service_config)?;
        let instances = service_config.instances.clone();
        for (instance_name, instance_value) in instances.into_iter() {
            handle_instance(
                instance_name,
                instance_value,
                &tar_file,
                ssh_config,
                &service_config,
                &image_name,
                &session
            )?;

        }
    }

    Ok(())
}


fn handle_instance(
    instance_name: Value,
    instance_value: Value,
    tar_file: &str,
    ssh_config: &SSHConfig,
    service_config: &ServiceConfig,
    image_name: &str,
    session: &Session
) -> anyhow::Result<()> {

    let cmd = resolve_instace_command(
        &instance_name,
        &instance_value,
        service_config,
        image_name
    )?;

    dbg!(&instance_name);

    scp_send(
        &tar_file,
        &format!("/tmp/{}", tar_file),
        ssh_config,
    )?;

    docker_load_and_run(
        &session,
        &format!("/tmp/{}", tar_file),
        cmd,
        &instance_name.as_str().unwrap(),
        &ssh_config
    )?;

    check_instance(
        &instance_name.as_str().unwrap(),
        instance_value,
        ssh_config
    );

    Ok(())

}


fn check_instance(instance_name: &str, instance_value: Value, ssh_config: &SSHConfig) {
    // ✅ Check se a instância subiu
    if let Some(instance_map) = instance_value.as_mapping() {
        if let Some(check) = instance_map.get(&Value::String("check".to_string())) {
            if let Some(check_map) = check.as_mapping() {
                let port = check_map.get(&Value::String("port".to_string()))
                    .and_then(|v| v.as_u64())
                    .expect("Check port não definido");
                let endpoint = check_map.get(&Value::String("endpoint".to_string()))
                    .and_then(|v| v.as_str())
                    .expect("Check endpoint não definido");

                let url = format!("{}:{}{}", ssh_config.host, port, endpoint);
                let client = Client::new();

                // Espera até 30s pelo container
                let mut success = false;
                for _ in 0..30 {
                    if let Ok(resp) = client.get(&url).send() {
                        if resp.status().is_success() {
                            success = true;
                            break;
                        }
                    }
                    thread::sleep(Duration::from_secs(1));
                }

                if !success {
                    panic!(
                        "A instância {} não respondeu no endpoint {}",
                        instance_name,
                        url
                    );
                }

                println!(
                    "Instância {} ok em {}",
                    instance_name,
                    url
                );
            }
        }
    }
}



fn resolve_instace_command(
    instance_name: &Value,
    instance_value: &Value,
    service_config: &ServiceConfig,
    image_name: &str,
) -> anyhow::Result<String> {

    let network_mode: Option<String> = service_config.network_mode.clone();
    let restart: Option<String> = service_config.restart.clone();
    let env_file: Option<Vec<String>> = service_config.env_file.clone();
    let volumes: Option<Vec<String>> = service_config.volumes.clone();
    let mut environment: Option<Vec<String>> = service_config.environment.clone();
    let _depends_on: Option<Vec<String>> = service_config.depends_on.clone();
    let mut main_command: Option<String> = None;

    let instance_name: String = from_value(instance_name.clone())?;
    let config: ContainerConfig = from_value(instance_value.clone())?;

    if let Some(v) = config.environment.clone() { 
        environment = Some(v);
    }

    if let Some(v) = config.command.clone() {
        main_command = Some(v);
    }

    let mut cmd = format!("docker run -d --name {}", instance_name);

    if let Some(ref net) = network_mode {
        cmd += &format!(" --network {}", net);
    }

    if let Some(ref r) = restart {
        cmd += &format!(" --restart {}", r);
    }

    if let Some(ref env_files) = env_file {
        for f in env_files {
            cmd += &format!(" --env-file {}", f);
        }
    }

    if let Some(ref envs) = environment {
        for e in envs {
            cmd += &format!(" -e {}", e);
        }
    }

    if let Some(ref vols) = volumes {
        for v in vols {
            cmd += &format!(" -v {}", v);
        }
    }

    cmd += &format!(" {}", image_name);
    if let Some(ref command) = main_command {
        cmd += &format!(" {}", command);
    }


    Ok(cmd)

}

