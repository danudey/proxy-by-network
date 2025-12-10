mod network_manager;
mod active_connections;

use zbus::Connection;

use std::io::Read;
use std::fs::OpenOptions;
use std::process::exit;
use std::io::IsTerminal;

use toml::Table;

use log::{error, info, LevelFilter};
use systemd_journal_logger::JournalLog;

use crate::network_manager::NetworkManagerProxy;
use crate::active_connections::ActiveConnectionProxy;

const CONFIG_FILE_GLOBAL: &str ="/etc/proxy-by-network.toml";

fn get_config(network_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("net.cdslash.proxy-by-net");

    let config_path_user = xdg_dirs
        .place_config_file("config.toml").unwrap_or("".into());
    let mut config_data = String::new();

    // let config_file_paths = [config_path, "/etc/proxy-by-network.toml"];

    let config_file_user = OpenOptions::new()
            .read(true)
            .open(config_path_user.clone());
    let config_file_global = OpenOptions::new()
            .read(true)
            .open(CONFIG_FILE_GLOBAL);

    let mut config_file = if config_file_user.is_ok() {
        info!("Loading config file {}", config_path_user.to_str().unwrap());
        config_file_user.unwrap()
    } else {
        info!("Loading config file {}", CONFIG_FILE_GLOBAL);
        config_file_global.unwrap()
    };

    // if config_file.is_err() {
    //     error!("Unable to load config file: {}", config_file.err().unwrap());
    //     return Err("Unable to load config file")?;
    // }

    let _ = config_file.read_to_string(&mut config_data);

    let config = config_data.parse::<Table>().unwrap();
    let net_config = config.get(&network_id);

    if net_config.is_none() {
        error!("Could not find proxy configuration for network {network_id}");
        exit(0);
    } else {
        let x = net_config.unwrap().get("http").unwrap();
        let proxy_config = &x.as_str().unwrap().to_owned();
        info!("Network '{network_id}' has proxy configuration '{proxy_config}'");
        println!("{proxy_config}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::io::stdout().is_terminal() {
        colog::default_builder()
        .default_format()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();
    } else {
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .with_syslog_identifier(env!("CARGO_PKG_NAME").to_string())
            .install().unwrap();
        log::set_max_level(LevelFilter::Debug);
    }


    let connection = Connection::system().await?;
    let network_manager = NetworkManagerProxy::new(&connection).await?;
    let primary_connection = network_manager.primary_connection().await?;

    let ac = ActiveConnectionProxy::new(&connection, primary_connection).await?;
    let network_id = ac.id().await?;

    info!("Got network ID '{network_id}' from d-bus");

    drop(network_manager);
    drop(ac);

    get_config(network_id)

}
