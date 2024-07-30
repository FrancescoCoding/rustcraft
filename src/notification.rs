use notify_rust::Notification;

pub fn trigger_notification(success: bool, error_message: Option<&str>) {
    if success {
        if let Err(e) = Notification::new()
            .appname("RustCraft")
            .summary("Backup Completed")
            .body("Your Minecraft worlds have been successfully saved.")
            .icon("./assets/icon.ico")
            .show()
        {
            eprintln!("Failed to show notification: {:?}", e);
        }
    } else if let Some(msg) = error_message {
        if let Err(e) = Notification::new()
            .appname("RustCraft")
            .summary("Backup Error")
            .body(msg)
            .icon("./assets/error.png")
            .show()
        {
            eprintln!("Failed to show notification: {:?}", e);
        }
    }
}
