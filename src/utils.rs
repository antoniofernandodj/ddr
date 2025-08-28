
use std::fs;
use std::io::Read;

use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use dotenvy::from_filename;
use std::env;
use regex::Regex;
use serde_yaml::{from_str, Value};
use std::collections::HashMap;

use crate::models::SSHConfig;


pub fn docker_save(image: &str, output_file: &str) -> anyhow::Result<()> {
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


pub fn scp_send(
    local_file: &str,
    remote_path: &str,
    ssh_config: &SSHConfig,
) -> anyhow::Result<()> {
    println!("Enviando o arquivo {local_file}");
    let session = get_session(ssh_config)?;

    let mut remote_file = session.scp_send(
        Path::new(remote_path),
        0o644,
        fs::metadata(local_file)?.len(),
        None
    )?;

    let mut local_file = fs::File::open(local_file)?;
    std::io::copy(&mut local_file, &mut remote_file)?;
    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;
    println!("Enviado!");
    Ok(())
}


pub fn docker_load_and_run(
    session: &Session,
    remote_file: &str,
    cmd: String,
    container_name: &str,
    ssh_config: &SSHConfig
) -> anyhow::Result<()> {
    println!("Docker load and run: {remote_file}");

    run_remote(session, &format!("docker load -i {}", remote_file))?;
    run_remote(session, &format!("docker rm -f {} || true", container_name))?;
    run_remote(session, &format!("cd {} && {}", ssh_config.from_dir, cmd))?;

    Ok(())
}


pub fn remove_local_and_remote_file(
    session: &Session,
    remote_file: &str
) -> anyhow::Result<()> {

    println!("Removendo arquivo local e remoto {remote_file}");
    run_remote(session, &format!("rm -f {}", remote_file))?;
    std::fs::remove_file(remote_file).ok();

    Ok(())
}


pub fn docker_run(session: &Session, cmd: String) -> anyhow::Result<()> {
    run_remote(session, &format!("{cmd}"))?;
    Ok(())
}


pub fn run_remote(session: &ssh2::Session, command: &str) -> anyhow::Result<()> {
    println!("Executando comando remoto:");
    dbg!(command);

    let mut channel = session.channel_session()?;
    channel.exec(command)?;

    // stdout
    let mut stdout = Vec::new();
    channel.read_to_end(&mut stdout)?;
    let stdout_str = String::from_utf8_lossy(&stdout);
    if !stdout_str.trim().is_empty() {
        println!("[remote stdout] {}", stdout_str);
    }

    // stderr
    let mut stderr = Vec::new();
    channel.stderr().read_to_end(&mut stderr)?;
    let stderr_str = String::from_utf8_lossy(&stderr);
    if !stderr_str.trim().is_empty() {
        eprintln!("[remote stderr] {}", stderr_str);
    }

    channel.wait_close()?;
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        anyhow::bail!("Comando remoto falhou ({exit_status}): {command}");
    }

    Ok(())
}


pub fn get_session(ssh_config: &SSHConfig) -> anyhow::Result<Session> {

    // Conexão TCP
    let tcp = TcpStream::connect(format!("{}:22", ssh_config.host))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Autenticação
    session.userauth_password(&ssh_config.user, &ssh_config.password)?;

    Ok(session)
}


pub fn get_ssh_config() -> anyhow::Result<SSHConfig> {
    // carrega o .env (se existir)
    from_filename("infra.secrets.env").ok();

    let user = env::var("SSH_USER")?;
    let host = env::var("SSH_HOST")?;
    let password = env::var("SSH_PASSWORD")?;
    let from_dir = env::var("DIR")?;
    
    Ok(SSHConfig::new(user, host, password, from_dir))
}


pub fn parse_variables(yaml_content: &str) -> anyhow::Result<HashMap<String, String>> {
    let raw_yaml: Value = from_str(yaml_content)?;
    
    // Se não tiver `define`, retorna HashMap vazio
    let define_values = match raw_yaml
        .get("define")
        .and_then(Value::as_mapping) {
            Some(values) => values,
            None => return Ok(HashMap::new()),
    };
    
    let mut variables = HashMap::new();
    for (key, value) in define_values {
        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
            variables.insert(k.to_string(), v.to_string());
        }
    }
    
    Ok(variables)
}


fn replace_variables(yaml_content: &str, variables: &HashMap<String, String>) -> anyhow::Result<String> {
    let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
    let mut replaced_content = yaml_content.to_string();

    for captures in re.captures_iter(yaml_content) {
        let full_match = captures.get(0).unwrap().as_str();
        let var_name = captures.get(1).unwrap().as_str();

        if let Some(value) = variables.get(var_name) {
            replaced_content = replaced_content.replace(full_match, value);
        } else {
            return Err(anyhow::anyhow!("Variável não encontrada: {}", var_name));
        }
    }
    
    Ok(replaced_content)
}


pub fn process_deployment_file(file_path: &str) -> anyhow::Result<Value> {
    let original_content = fs::read_to_string(file_path)?;

    let variables: HashMap<String, String> = parse_variables(
        &original_content
    )?;

    let processed_content = replace_variables(
        &original_content,
        &variables
    )?;

    let full_config: Value = from_str(&processed_content)?;
    
    Ok(full_config)
}
