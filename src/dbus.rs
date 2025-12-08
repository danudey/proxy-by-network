// use std::cmp::Ordering;
// use std::collections::HashMap;
// use std::fmt::{Debug, Display, Formatter};
// use tokio::time::{sleep, Duration};
// use zbus::export::serde::{Deserialize, Serialize};
// use zbus::zvariant::{OwnedObjectPath, Type};
// use zbus::{Connection, Proxy};

// use getset::Getters;

// use crate::networkmanager_generated::NetworkManagerProxy;
// use crate::notifications::NotificationService;
// use crate::wifi::WifiError;
// use crate::wifi::WiFiNetwork;
// use crate::{
//   NETWORKMANAGER_ACCESS_POINT_INTERFACE_ADDRESS, NETWORKMANAGER_ADDRESS,
//   NETWORKMANAGER_DEVICE_INTERFACE_ADDRESS, NETWORKMANAGER_WIRELESS_INTERFACE_ADDRESS,
// };

// #[derive(Serialize, Deserialize, Type, Debug, Getters)]
// pub struct DBusAccessPoint {
//   #[getset(get = "pub")]
//   ssid: String,
//   #[getset(get = "pub")]
//   bssid: String,
//   #[getset(get = "pub")]
//   strength: u8,
// }

// impl DBusAccessPoint {
//   pub fn new(ssid: String, bssid: String, strength: u8) -> Self {
//     Self {
//       ssid,
//       bssid,
//       strength,
//     }
//   }
// }

// impl WiFiNetwork for DBusAccessPoint {
//   fn ssid(&self) -> &str {
//     &self.ssid
//   }

//   fn bssid(&self) -> &str { &self.bssid }

//   fn strength(&self) -> u8 { self.strength }
// }

// impl Display for DBusAccessPoint {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     write!(f, "{} {} | {}", self.strength, self.bssid, self.ssid)
//   }
// }

// impl PartialEq<Self> for DBusAccessPoint {
//   fn eq(&self, other: &Self) -> bool {
//     self.strength < other.strength
//   }
// }

// impl Eq for DBusAccessPoint {}

// impl Ord for DBusAccessPoint {
//   fn cmp(&self, other: &Self) -> Ordering {
//     self.strength.cmp(&other.strength)
//   }
// }

// impl PartialOrd for DBusAccessPoint {
//   fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//     self.strength.partial_cmp(&other.strength)
//   }
// }

// /// Tries to identify a valid Wi-Fi device
// /// # Arguments
// /// * `dbus_connection` - a DBus system connection
// /// * `notification_service` - a notification channel
// /// # Returns
// /// A `Result` containing, either an instance of `OwnedObjectPath` or an error
// /// # Errors
// /// When no or more than one Wi-Fi devices are found
// pub async fn find_wifi_device(
//   dbus_connection: &Connection,
//   _notification_service: &NotificationService<'_>,
// ) -> Result<OwnedObjectPath, Box<dyn std::error::Error>> {
//   let network_manager_proxy = NetworkManagerProxy::new(dbus_connection).await?;
//   let all_network_devices = network_manager_proxy.get_all_devices().await?;
//   drop(network_manager_proxy);

//   let mut wireless_device_path: Option<OwnedObjectPath> = None;

//   for device in all_network_devices {
//     let device_proxy = Proxy::new(
//       dbus_connection,
//       NETWORKMANAGER_ADDRESS,
//       &device,
//       NETWORKMANAGER_DEVICE_INTERFACE_ADDRESS,
//     )
//     .await?;

//     match device_proxy.get_property::<u32>("DeviceType").await? {
//       2 => {
//         if wireless_device_path.as_ref().is_none() {
//           // 2 => NM_DEVICE_TYPE_WIFI
//           wireless_device_path = Some(device)
//         } else {
//           // notification_service
//           //   .notify_err("More than one wireless device detected. Only one supported.")
//           return Err(WifiError::MultipleWiFiDevices.into());
//         }
//       }
//       _ => continue,
//     }
//   }

//   match wireless_device_path {
//     None => Err(Box::new(WifiError::NoWiFiDevices)),
//     Some(val) => Ok(val),
//   }
// }


// /// Lists available Wi-Fi networks as a `Vec<DBusAccessPoint>`
// /// # Arguments
// /// * A DBus `device_path`
// /// * A `dbus_connection`
// /// * Whether to scan before listing
// /// # Returns
// /// A `Vec<DBusAccessPoint>`
// /// # Errors
// /// Returns an error type implementing std::error::Error if:
// /// * unable to list access points
// /// * unable to get the `Ssid` or `Strength` properties from the DBus proxy
// pub async fn list_available_networks(
//   device_path: &OwnedObjectPath,
//   dbus_connection: &Connection,
//   scan_first: bool,
// ) -> Result<Vec<DBusAccessPoint>, Box<dyn std::error::Error>> {
//   let device_proxy: Proxy<'_> = Proxy::new(
//     dbus_connection,
//     NETWORKMANAGER_ADDRESS,
//     device_path,
//     NETWORKMANAGER_WIRELESS_INTERFACE_ADDRESS,
//   )
//   .await?;

//   if scan_first {
//     scan(device_path, dbus_connection).await?;
//     sleep(Duration::from_millis(3000)).await;
//   }

//   let network_objects: Vec<OwnedObjectPath> = device_proxy.call("GetAccessPoints", &()).await?;
//   let mut ssids: Vec<DBusAccessPoint> = vec![];

//   for object_path in network_objects {
//     let access_point_proxy = Proxy::new(
//       dbus_connection,
//       NETWORKMANAGER_ADDRESS,
//       object_path,
//       NETWORKMANAGER_ACCESS_POINT_INTERFACE_ADDRESS,
//     )
//     .await?;

//     let ssid = access_point_proxy.get_property::<Vec<u8>>("Ssid").await?;
//     let bssid = access_point_proxy
//       .get_property::<String>("HwAddress")
//       .await?;
//     let strength = access_point_proxy.get_property::<u8>("Strength").await?;

//     ssids.push(DBusAccessPoint::new(
//       String::from_utf8(ssid).unwrap(),
//       bssid,
//       strength,
//     ))
//   }

//   Ok(ssids)
// }


// /// Initiates a Wi-Fi network scan on the specified device.
// ///
// /// This function sends a scan request to NetworkManager via DBus to discover
// /// available wireless networks in range of the specified wireless device.
// ///
// /// # Arguments
// /// * `device_path` - The DBus object path of the wireless device to scan
// /// * `dbus_connection` - Active DBus connection for communicating with NetworkManager
// ///
// /// # Returns
// /// A `zbus::Result<()>` indicating success or failure of the scan request.
// ///
// /// # Errors
// ///
// /// Returns `zbus::Error` if:
// /// - The DBus proxy cannot be created
// /// - The scan request fails
// /// - NetworkManager is not available
// /// - The specified device path is invalid
// pub async fn scan(
//   device_path: &OwnedObjectPath,
//   dbus_connection: &Connection,
// ) -> zbus::Result<()> {
//   let device_proxy: Proxy<'_> = Proxy::new(
//     dbus_connection,
//     NETWORKMANAGER_ADDRESS,
//     device_path,
//     NETWORKMANAGER_WIRELESS_INTERFACE_ADDRESS,
//   )
//   .await?;

//   let scan_options: HashMap<String, zbus::zvariant::Value> = HashMap::new();
//   // scan_options.insert(String::from("SSID"), zbus::zvariant::Value::new(true));

//   device_proxy.call("RequestScan", &scan_options).await
// }

// /// Sorts and deduplicates networks whose referente is passed
// pub fn unique_access_points(list: &mut Vec<DBusAccessPoint>) {
//   list.sort_by(|this, other| other.ssid.cmp(&this.ssid));
//   list.dedup_by(|this, other| other.ssid.eq(&this.ssid) && !(other.strength.eq(&this.strength)));
//   list.sort();
// }
