use std::env;
use systemstat::{Platform, System};

mod command;
mod components;
mod constants;
use components::banner::disp_banner;
use components::docker::disp_docker;
use components::fail_2_ban::disp_fail_2_ban;
use components::filesystem::disp_filesystem;
use components::last_login::disp_last_login;
use components::last_run::disp_last_run;
use components::memory::disp_memory;
use components::service_status::disp_service_status;
use components::ssl_certs::disp_ssl;
use components::uptime::disp_uptime;
use components::weather::disp_weather;
mod config;
use config::{get_config, ComponentConfig, Config};

async fn print_motd(config: Config) {
    let sys = System::new();
    let mut bar_size_hint: Option<usize> = None;

    for component_config in config.components {
        match component_config {
            ComponentConfig::Banner(banner_config) => {
                disp_banner(banner_config).unwrap_or_else(|err| println!("Banner error: {}", err));
                println!();
            }
            ComponentConfig::Docker(docker_config) => {
                println!("Docker:");
                disp_docker(docker_config)
                    .await
                    .unwrap_or_else(|err| println!("Docker status error: {}", err));
                println!();
            }
            ComponentConfig::Fail2Ban(fail_2_ban_config) => {
                disp_fail_2_ban(fail_2_ban_config)
                    .unwrap_or_else(|err| println!("Fail2Ban error: {}", err));
                println!();
            }
            ComponentConfig::Filesystems(filesystems_config) => {
                bar_size_hint = disp_filesystem(filesystems_config, &config.global, &sys)
                    .unwrap_or_else(|err| {
                        println!("Filesystem error: {}", err);
                        None
                    });
                println!();
            }
            ComponentConfig::LastLogin(last_login_config) => {
                disp_last_login(last_login_config, &config.global)
                    .unwrap_or_else(|err| println!("Last login error: {}", err));
                println!();
            }
            ComponentConfig::LastRun(last_run_config) => {
                disp_last_run(last_run_config, &config.global)
                    .unwrap_or_else(|err| println!("Last run error: {}", err));
            }
            ComponentConfig::Memory(memory_config) => {
                disp_memory(memory_config, &config.global, &sys, bar_size_hint) // TODO:
                    .unwrap_or_else(|err| println!("Memory error: {}", err));
                println!();
            }
            ComponentConfig::ServiceStatus(service_status_config) => {
                println!("System Services:");
                disp_service_status(service_status_config, false)
                    .unwrap_or_else(|err| println!("Service status error: {}", err));
                println!();
            }
            ComponentConfig::UserServiceStatus(user_service_status_config) => {
                println!("User Services:");
                disp_service_status(user_service_status_config, true)
                    .unwrap_or_else(|err| println!("User service status error: {}", err));
                println!();
            }
            ComponentConfig::SSLCerts(ssl_certificates_config) => {
                disp_ssl(ssl_certificates_config, &config.global)
                    .unwrap_or_else(|err| println!("SSL Certificate error: {}", err));
                println!();
            }
            ComponentConfig::Uptime(uptime_config) => {
                disp_uptime(uptime_config, &sys)
                    .unwrap_or_else(|err| println!("Uptime error: {}", err));
                println!();
            }
            ComponentConfig::Weather(weather_config) => {
                disp_weather(weather_config)
                    .unwrap_or_else(|err| println!("Weather error: {}", err));
                println!();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args();

    match get_config(args) {
        Ok(config) => {
            print_motd(config).await;
        }
        Err(e) => println!("Config Error: {}", e),
    }
    Ok(())
}
