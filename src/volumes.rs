use serde_yaml::{from_value, Mapping, Value};

use crate::{
    models::SSHConfig,
    utils::{docker_run, get_session}
};


pub fn handle_volumes(
    ssh_config: &SSHConfig,
    deploy_map: &Mapping,
    dry_run: bool
) -> anyhow::Result<()> {

    // Pega o grupo "volumes" como Value e garante que é Mapping
    let group_config: Mapping = deploy_map
        .get(&Value::String("volumes".to_string()))
        .cloned()
        .expect("Group config 'volumes' não encontrado!")
        .as_mapping()
        .expect("'volumes' não é um mapping")
        .to_owned(); // agora é Mapping, iterável

    let session = get_session(ssh_config)?;

    for (volume_name, volume_value) in group_config {
        let volume_name: String = from_value(volume_name).unwrap();

        let mut cmd = format!("docker volume create {}", volume_name);

        if let Some(volume_map) = volume_value.as_mapping() {
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

        if !dry_run {
            docker_run(&session, cmd)?;
        }
    }

    Ok(())
}
