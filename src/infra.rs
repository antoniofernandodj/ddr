use serde_yaml::{
    from_value,
    Value
};
use crate::{
    models::{
        InfraConfig,
        SSHConfig
    },
    utils::{
        self,
        docker_load_and_run,
        docker_save,
        scp_send
    }
};


pub fn handle_infra(
    ssh_config: &SSHConfig,
    group_config: Value
) -> anyhow::Result<()> {
    let session = utils::get_session(ssh_config)?;
    let group_config = group_config
        .as_mapping()
        .unwrap()
        .to_owned();

    for (image_name, infra_value) in group_config {
        let image_name: String = from_value(image_name)?;
        let tar_file = format!("{}.tar", image_name.replace("/", "_").replace(":", "_"));
        docker_save(&image_name, &tar_file)?;


        let infra_config: InfraConfig = from_value(infra_value).unwrap();

        for (instance_name, instance_value) in infra_config.instances {
            scp_send(
                &tar_file,
                &format!("/tmp/{}", tar_file),
                ssh_config,
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
                cmd_str,
                &instance_name,
                ssh_config
            )?;
        }
    }

    Ok(())
}

