#[allow(dead_code, unused)]

use std::fs;

use serde::Deserialize;
use serde_yaml::{from_value, Mapping, Value};
use clap::{Parser, Subcommand};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;


#[derive(Debug, Deserialize)]
struct ContainerConfig {

    network_mode: Option<String>,
    restart: Option<String>,
    env_file: Option<Vec<String>>,
    volumes: Option<Vec<String>>,
    environment: Option<Vec<String>>,
    depends_on: Option<Vec<String>>,

    command: Option<String>,
}


#[derive(Debug, Deserialize)]
struct ServiceConfig {
    network_mode: Option<String>,
    restart: Option<String>,
    env_file: Option<Vec<String>>,
    volumes: Option<Vec<String>>,
    environment: Option<Vec<String>>,
    depends_on: Option<Vec<String>>,
    instances: Mapping,
}


#[derive(Debug, Deserialize)]
struct InfraConfig {
    network_mode: Option<String>,
    restart: Option<String>,
    env_file: Option<Vec<String>>,
    volumes: Option<Vec<String>>,
    environment: Option<Vec<String>>,
    depends_on: Option<Vec<String>>,
    mem_limit: Option<String>,
    healthcheck: Option<Value>, // Pode ser mapeado melhor depois
    command: Option<String>,
    instances: Mapping, // Algumas infra pode ter instâncias
}





#[derive(Parser)]
#[command(name = "ddr")]
#[command(about = "CLI para gerar comandos Docker a partir do deploy.yaml", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Deploy {
        #[arg(value_enum)]
        target: DeployTarget,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum DeployTarget {
    Services,
    Infra,
    Volumes,
    Networks,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Lê o deploy.yaml
    let deploy_yaml = read_deployment_file("deploy.yaml")?;
    let deploy_map = deploy_yaml.as_mapping().unwrap();

    match cli.command {
        Commands::Deploy { target } => match target {
            DeployTarget::Services => {
                if let Some(group) = deploy_map.get(
                    &Value::String("services".to_string())
                ) {
                    handle_services(group.clone())?;
                }
            }
            DeployTarget::Infra => {
                if let Some(group) = deploy_map.get(
                    &Value::String("infra".to_string())
                ) {
                    handle_infra(group.clone())?;
                }
            }
            DeployTarget::Volumes => {
                if let Some(group) = deploy_map.get(
                    &Value::String("volumes".to_string())
                ) {
                    handle_volumes(group.clone());
                }
            }
            DeployTarget::Networks => {
                if let Some(group) = deploy_map.get(&Value::String("networks".to_string())) {
                    handle_networks(group.clone());
                }
            }
        },
    }

    Ok(())
}



fn read_deployment_file(path: &str) -> anyhow::Result<Value> {
    let content: String = fs::read_to_string(path)?;
    let config: Value = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn handle_services(group_config: Value) -> anyhow::Result<()> {
    // SSH para rodar docker load e run
    let session = get_session()?;
    let group_config = group_config
        .as_mapping()
        .unwrap()
        .to_owned();

    for (image_name, service_config) in group_config {
        let image_name: String = from_value(image_name)?;
        let tar_file = format!("{}.tar", image_name.replace("/", "_"));
        docker_save(&image_name, &tar_file)?;

        let service_config: ServiceConfig = from_value(service_config)?;

        let network_mode = service_config.network_mode.clone();
        let restart = service_config.restart.clone();
        let env_file = service_config.env_file.clone();
        let volumes = service_config.volumes.clone();
        let mut environment = service_config.environment.clone();
        let depends_on = service_config.depends_on.clone();
        let mut main_command: Option<String> = None;

        for (instance_name, instance_value) in service_config.instances {
            scp_send(
                "user",
                "host",
                &tar_file,
                &format!("/tmp/{}", tar_file)
            )?;

            let instance_name: String = from_value(instance_name)?;
            let config: ContainerConfig = from_value(instance_value)?;

            // Sobrescreve se houver configuração na instância
            if let Some(v) = config.environment.clone() { environment = Some(v); }
            if let Some(v) = config.command.clone() { main_command = Some(v); }

            // Começa a montar o docker run
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

            // Adiciona a imagem
            cmd += &format!(" {}", image_name);

            // Adiciona o comando da instância se existir
            if let Some(ref command) = main_command {
                cmd += &format!(" {}", command);
            }

            docker_load_and_run(
                &session,
                &format!("/tmp/{}", tar_file),
                cmd
            )?;

        }
    }

    Ok(())
}



fn handle_infra(group_config: Value) -> anyhow::Result<()> {
    let session = get_session()?;
    let group_config = group_config
        .as_mapping()
        .unwrap()
        .to_owned();

    for (image_name, infra_value) in group_config {
        let image_name: String = from_value(image_name)?;
        let tar_file: String = format!("{}.tar", image_name.replace("/", "_"));
        docker_save(&image_name, &tar_file)?;


        let infra_config: InfraConfig = from_value(infra_value).unwrap();

        for (instance_name, instance_value) in infra_config.instances {
            scp_send(
                "user",
                "host",
                &tar_file,
                &format!("/tmp/{}", tar_file)
            )?;

            let instance_name: String = from_value(instance_name.clone()).unwrap();
            let mut cmd_str = format!("docker run -d --name {}", instance_name);
            // network_mode
            if let Some(ref net) = infra_config.network_mode {
                cmd_str += &format!(" --network {}", net);
            }
            // restart
            if let Some(ref r) = infra_config.restart {
                cmd_str += &format!(" --restart {}", r);
            }
            // env_file
            if let Some(ref env_files) = infra_config.env_file {
                for f in env_files {
                    cmd_str += &format!(" --env-file {}", f);
                }
            }
            // environment
            if let Some(ref envs) = infra_config.environment {
                for e in envs {
                    cmd_str += &format!(" -e {}", e);
                }
            }
            // mem_limit
            if let Some(ref mem) = infra_config.mem_limit {
                cmd_str += &format!(" --memory {}", mem);
            }
            // volumes
            if let Some(ref vols) = infra_config.volumes {
                for v in vols {
                    cmd_str += &format!(" -v {}", v);
                }
            }
            // comando específico da instância
            if let Some(instance_map) = instance_value.as_mapping() {
                if let Some(cmd) = instance_map.get(
                    &Value::String("command".to_string())
                ) {
                    if let Some(cmd_str_val) = cmd.as_str() {
                        cmd_str += &format!(" {}", cmd_str_val);
                    }
                }
                // environment específico da instância
                if let Some(env) = instance_map.get(
                    &Value::String("environment".to_string())
                ) {
                    if let Some(env_seq) = env.as_sequence() {
                        for e in env_seq {
                            if let Some(e_str) = e.as_str() {
                                cmd_str += &format!(" -e {}", e_str);
                            }
                        }
                    }
                }
                // volumes específicos da instância
                if let Some(vols) = instance_map.get(
                    &Value::String("volumes".to_string())
                ) {
                    if let Some(vol_seq) = vols.as_sequence() {
                        for v in vol_seq {
                            if let Some(v_str) = v.as_str() {
                                cmd_str += &format!(" -v {}", v_str);
                            }
                        }
                    }
                }
            }
            cmd_str += &format!(" {}", image_name);

            docker_load_and_run(
                &session,
                &format!("/tmp/{}", tar_file),
                cmd_str
            )?;
        }
    }

    Ok(())
}


fn handle_volumes(group_config: Value) -> anyhow::Result<()> {
    let group_config = group_config.as_mapping().unwrap().to_owned();
    let session = get_session()?;

    for (volume_name, volume_value) in group_config {
        let volume_name: String = from_value(volume_name).unwrap();

        // Pode ter opções como driver, driver_opts etc.
        let mut cmd = format!("docker volume create {}", volume_name);

        if let Some(volume_map) = volume_value.as_mapping() {
            // driver
            if let Some(driver) = volume_map.get(
                &Value::String("driver".to_string())
            ) {
                if let Some(driver_str) = driver.as_str() {
                    cmd += &format!(" --driver {}", driver_str);
                }
            }
            // driver_opts
            if let Some(driver_opts) = volume_map.get(
                &Value::String("driver_opts".to_string())
            ) {
                if let Some(opts_map) = driver_opts.as_mapping() {
                    for (k, v) in opts_map {
                        let k: String = from_value(k.clone()).unwrap();
                        let v: String = from_value(v.clone()).unwrap();
                        cmd += &format!(" --opt {}={}", k, v);
                    }
                }
            }
        }

        println!("\n{}", cmd);
        docker_run(&session, cmd)?;
    }

    Ok(())
}


fn handle_networks(group_config: Value) {
    let group_config = group_config.as_mapping().unwrap().to_owned();

    for (network_name, network_value) in group_config {
        let network_name: String = from_value(network_name).unwrap();

        let mut cmd = format!("docker network create {}", network_name);

        if let Some(network_map) = network_value.as_mapping() {
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
    }
}

fn docker_save(image: &str, output_file: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("docker")
        .arg("save")
        .arg("-o")
        .arg(output_file)
        .arg(image)
        .status()?;

    if !status.success() {
        anyhow::bail!("Erro ao exportar a imagem {}", image);
    }

    Ok(())
}



fn scp_send(
    user: &str,
    host: &str,
    local_file: &str,
    remote_path: &str
) -> anyhow::Result<()> {
    let tcp = TcpStream::connect(format!("{}:22", host))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.userauth_agent(user)?;
    
    let mut remote_file = session.scp_send(
        Path::new(remote_path),
        0o644,
        fs::metadata(local_file)?.len(),
        None
    )?;

    let mut local_file = fs::File::open(local_file)?;
    std::io::copy(&mut local_file, &mut remote_file)?;
    
    Ok(())
}


fn docker_load_and_run(
    session: &Session,
    remote_file: &str,
    cmd: String
) -> anyhow::Result<()> {
    let mut channel = session.channel_session()?;
    channel.exec(&format!("docker load -i {}", remote_file))?;
    channel.wait_close()?;

    let mut run_channel = session.channel_session()?;
    run_channel.exec(&cmd)?;
    run_channel.wait_close()?;

    Ok(())
}

fn docker_run(
    session: &Session,
    cmd: String
) -> anyhow::Result<()> {
    let mut run_channel = session.channel_session()?;
    run_channel.exec(&cmd)?;
    run_channel.wait_close()?;

    Ok(())
}


fn get_session() -> anyhow::Result<Session> {
    // SSH para rodar docker load e run
    let tcp = TcpStream::connect("host:22")?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.userauth_agent("user")?;

    return Ok(session)
}
