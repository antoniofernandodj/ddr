use std::{collections::HashSet, thread, time::Duration};

use serde_yaml::{from_value, Mapping, Value};
use ssh2::Session;
use reqwest::blocking::Client;
use crate::{
    models::{
        ContainerConfig,
        HealthCheck,
        RemoteHealthCheck,
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


pub fn handle_group(
    ssh_config: &SSHConfig,
    group_name: &str,
    deploy_map: &Mapping,
    dry_run: bool,
) -> anyhow::Result<()> {

    let group_config: Value = deploy_map
        .get(&Value::String(group_name.to_string()))
        .cloned()
        .expect("Group config não encontrado!");

    let mut deployed_services: HashSet<String> = HashSet::new();
    let mut services_to_deploy = group_config
        .as_mapping()
        .expect("Group config não é um mapping")
        .to_owned();

    let session: Session = get_session(ssh_config)?;
    while !services_to_deploy.is_empty() {

        let ready_for_this_wave = resolve_this_wave(
            &services_to_deploy,
            &deployed_services
        )?;
    
        for image_name in ready_for_this_wave {
            let service_config = services_to_deploy
                .get(&image_name)
                .unwrap()
                .clone();

            let tar_file: String = format!(
                "{}.tar", &image_name.replace("/", "_").replace(":", "_")
            );

            println!("----------------- DEPLOY DE SERVICE: {image_name} -----------------");
            println!("Salvando imagem em tar file: {tar_file}");
            if !dry_run {
                docker_save(&image_name, &tar_file)?;
            }
            println!("Salvou a imagem {tar_file} em tar file");

            let service_config: ServiceConfig = from_value(service_config)?;
            
            let instances = service_config.instances.clone();

            if !dry_run {
                scp_send(
                    &tar_file,
                    &format!("/tmp/{}", tar_file),
                    ssh_config,
                )?;
            }

            for (instance_name, instance_value) in instances.into_iter() {
                let container_config: ContainerConfig = from_value(instance_value.clone())?;
                let instance_name = instance_name.as_str().unwrap();

                println!("---------- Deploy de instancia `{instance_name}` ----------");
                
                if !dry_run {
                    handle_instance(
                        instance_name,
                        container_config,
                        &tar_file,
                        ssh_config,
                        &service_config,
                        &image_name,
                        &session,
                    )?;
                }

            }
            
            if !dry_run {
                remove_local_and_remote_file(
                    &session,
                    &tar_file
                )?;
            }

            deployed_services.insert(image_name.clone());
            services_to_deploy.remove(image_name);
        }

    }

    Ok(())
}


fn resolve_this_wave(
    services_to_deploy: &Mapping,
    deployed_services: &HashSet<String>
) -> anyhow::Result<Vec<String>> {

    let mut ready_for_this_wave: Vec<String> = Vec::new();
    for (image_name, service_config) in services_to_deploy.iter() {
        let mut image_name = image_name
            .as_str()
            .expect("Nenhuma image econtrada")
            .to_owned();

        let service: ServiceConfig = from_value(service_config.clone())?;
        if let Some(image) =  service.image {
            image_name = image;
        }
        let dependencies = service.depends_on.clone().unwrap_or_default();
        let all_deps_ready = if dependencies.is_empty() { true } else {
            dependencies.iter().all(|dep| deployed_services.contains(dep))
        };
        if all_deps_ready {
            ready_for_this_wave
            .push(image_name);
        }
    }
    if ready_for_this_wave.is_empty() {
        return Err(
            anyhow::anyhow!(
                "Dependência cíclica ou impossível de satisfazer."
            )
        );
    }

    Ok(ready_for_this_wave)
}


fn handle_instance(
    instance_name: &str,
    container_config: ContainerConfig,
    tar_file: &str,
    ssh_config: &SSHConfig,
    service_config: &ServiceConfig,
    image_name: &str,
    session: &Session,
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

    if let Some(check_health) = &container_config.remotecheck {
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
    check_health: &RemoteHealthCheck,
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
    let mut cmd: String = format!("docker run -d --name {}", instance_name);

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

    if let Some(ref hc) = container_config.healthcheck {
        let cmd_string: String = build_health_cmd(hc);
        if !cmd_string.is_empty() {
            cmd = format!(
                "{} --health-cmd='{}' --health-interval={} --health-timeout={} --health-retries={}",
                cmd,
                cmd_string,
                hc.interval,
                hc.timeout,
                hc.retries
            );
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

    if let Some(ref mut check_container) = container_config.remotecheck {
        if let Some(ref check_service) = service_config.remotecheck {
            if let Some(port) = check_service.port {
                check_container.port = Some(port);
            }
        } else {
            if let Some(ref check_service) = service_config.remotecheck {
                if
                let Some(port) = check_service.port &&
                let Some(ref endpoint) = check_service.endpoint {
                    container_config.remotecheck = Some(
                        RemoteHealthCheck {
                            port: Some(port),
                            endpoint: Some(endpoint.clone())
                        }
                    )
                }
            }
        }
    }

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


fn build_health_cmd(hc: &HealthCheck) -> String {
    if hc.test.is_empty() {
        return String::new();
    }

    let cmd_parts: &[String] = if hc.test[0] == "CMD" || hc.test[0] == "CMD-SHELL" {
        &hc.test[1..]
    } else {
        &hc.test[..]
    };

    let cmd_string = cmd_parts
        .iter()
        .map(|s| {
            if s.contains(' ') {
                format!("\"{}\"", s)  // coloca aspas se tiver espaço
            } else {
                s.clone()
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    cmd_string
}
