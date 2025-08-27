use std::{thread, time::Duration};

use serde_yaml::{from_value, Value};
use ssh2::Session;
use reqwest::blocking::Client;
use crate::{
    models::{
        CheckHealth,
        ContainerConfig,
        SSHConfig,
        ServiceConfig
    },
    utils::{
        docker_load_and_run,
        docker_save,
        get_session,
        remove_local_and_remote_file,
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
        let tar_file: String = format!(
            "{}.tar", image_name.replace("/", "_").replace(":", "_")
        );

        println!("Salvando imagem em tar file: {tar_file}");
        docker_save(&image_name, &tar_file)?;
        println!("Salvou a imagem em uma tar file");

        let service_config: ServiceConfig = from_value(service_config)?;
        
        let instances = service_config.instances.clone();

        scp_send(
            &tar_file,
            &format!("/tmp/{}", tar_file),
            ssh_config,
        )?;

        for (instance_name, instance_value) in instances.into_iter() {

            let container_config: ContainerConfig = from_value(instance_value.clone())?;
            let instance_name = instance_name.as_str().unwrap();
    
            handle_instance(
                instance_name,
                container_config,
                &tar_file,
                ssh_config,
                &service_config,
                &image_name,
                &session
            )?;

        }

        remove_local_and_remote_file(
            &session,
            &tar_file
        )?;

    }

    Ok(())
}


fn handle_instance(
    instance_name: &str,
    container_config: ContainerConfig,
    tar_file: &str,
    ssh_config: &SSHConfig,
    service_config: &ServiceConfig,
    image_name: &str,
    session: &Session
) -> anyhow::Result<()> {

    let cmd: String = resolve_instace_command(
        &instance_name,
        &container_config,
        service_config,
        image_name
    )?;

    println!("Instance name: {instance_name}");

    docker_load_and_run(
        &session,
        &format!("/tmp/{}", tar_file),
        cmd,
        instance_name,
        &ssh_config
    )?;

    if let Some(check_health) = &container_config.check {
        check_instance(
            instance_name,
            check_health,
            ssh_config,
            &session,
            tar_file
        )?;
    }

    Ok(())

}


fn check_instance(
    instance_name: &str,
    check_health: &CheckHealth,
    ssh_config: &SSHConfig,
    session: &Session,
    remote_file: & str
) -> anyhow::Result<()> {

    if
    let Some(endpoint) = check_health.endpoint.clone() &&
    let Some(port) = check_health.port {

        let url: String = format!(
            "http://{}:{}{}",
            ssh_config.host,
            port,
            endpoint
        );
        
        let client: Client = Client::new();
        let mut success: bool = false;
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
            remove_local_and_remote_file(session, remote_file)?;
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

    Ok(())
}


fn resolve_instace_command(
    instance_name: &str,
    container_config: &ContainerConfig,
    service_config: &ServiceConfig,
    image_name: &str,
) -> anyhow::Result<String> {

    let container_config: ContainerConfig = resolve_instance_config_values(
        container_config,
        service_config
    )?;

    // Construir o comando principal
    let mut cmd = format!("docker run -d --name {}", instance_name);

    if let Some(ref net) = container_config.network_mode {
        cmd += &format!(" --network {}", net);
    }

    if let Some(ref r) = container_config.restart {
        cmd += &format!(" --restart {}", r);
    }

    if let Some(ref env_files) = container_config.env_file {
        for f in env_files {
            cmd += &format!(" --env-file {}", f);
        }
    }

    if let Some(ref envs) = container_config.environment {
        for e in envs {
            cmd += &format!(" -e {}", e);
        }
    }

    if let Some(ref vols) = container_config.volumes {
        for v in vols {
            cmd += &format!(" -v {}", v);
        }
    }

    cmd += &format!(" {}", image_name);
    if let Some(ref command) = container_config.command {
        cmd += &format!(" {}", command);
    }

    Ok(cmd)

}


fn resolve_instance_config_values(
    container_config: &ContainerConfig,
    service_config: &ServiceConfig,
) -> anyhow::Result<ContainerConfig> {

    let mut container_config: ContainerConfig = container_config.clone();

    if let Some(ref mut check_container) = container_config.check {
        if let Some(ref check_service) = service_config.check {
            if let Some(port) = check_service.port {
                check_container.port = Some(port);
            }
        } else {
            if let Some(ref check_service) = service_config.check {
                if
                let Some(port) = check_service.port &&
                let Some(ref endpoint) = check_service.endpoint {
                    container_config.check = Some(
                        CheckHealth {
                            port: Some(port),
                            endpoint: Some(endpoint.clone())
                        }
                    )
                }
            }
        }
    }

    container_config.depends_on = container_config.depends_on
        .or_else(|| service_config.depends_on.clone());

    container_config.environment = container_config.environment
        .or_else(|| service_config.environment.clone());

    container_config.network_mode = container_config.network_mode
        .or_else(|| service_config.network_mode.clone());

    container_config.restart = container_config.restart
        .or_else(|| service_config.restart.clone());

    container_config.env_file = container_config.env_file
        .or_else(|| service_config.env_file.clone());

    container_config.volumes = container_config.volumes
        .or_else(|| service_config.volumes.clone());

    return Ok(container_config);

}