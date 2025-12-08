mod network_manager;
mod activeconnections;
mod dbus;

use zbus::Connection;
use crate::network_manager::NetworkManagerProxy;
use crate::activeconnections::ActiveConnectionProxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let connection = Connection::system().await?;
    let network_manager = NetworkManagerProxy::new(&connection).await?;
    let primary_connection = network_manager.primary_connection().await?;
    let ac = ActiveConnectionProxy::new(&connection, primary_connection).await?;
    let network_id = ac.id().await?;

    if network_id.starts_with("TeamSweden") {
        println!("http://10.0.0.2:3142");
    }

    drop(network_manager);
    drop(ac);

    let xdg_dirs = xdg::BaseDirectories::with_prefix("myapp");

    Ok(())
}
