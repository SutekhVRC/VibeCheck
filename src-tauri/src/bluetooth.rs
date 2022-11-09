use btleplug::api::Manager as _;
use btleplug::platform::Manager;


pub async fn detect_btle_adapter() -> bool {
    if let Ok(manager) = Manager::new().await {
        if let Ok(adapters) = manager.adapters().await {
            return adapters.is_empty();
        } else {
            return false;
        }
    } else {
        println!("[-] Failed to create btle Manager.");
        return false;
    }
}