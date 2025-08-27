use serde_yaml::{from_value, Value};

use crate::{
    models::SSHConfig,
    utils::{docker_run, get_session}
};


pub fn handle_volumes(
    ssh_config: &SSHConfig,
    group_config: Value
) -> anyhow::Result<()> {
    let group_config = group_config.as_mapping().unwrap().to_owned();
    let session = get_session(ssh_config)?;

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
        docker_run(&session, cmd, ssh_config)?;
    }

    Ok(())
}
