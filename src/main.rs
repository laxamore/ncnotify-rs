use std::path::Path;
use std::process::Command;
use std::{thread, time};
use notify::{Watcher, RecursiveMode, Error, Event};
use config::Config;
use std::sync::Mutex;

static CHANGED_FILES: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[derive(Debug)]
struct NCNotifyConfig {
    ignore_path_monitor: Vec<String>,
    data_path: String,
    commands_to_run: String,
    notify_interval: u64,
}

// Load config.toml
fn load_config() -> NCNotifyConfig {
    let conf = Config::builder()
        .add_source(config::File::with_name("/etc/ncnotify/config.toml"))
        .build()
        .unwrap();

    return NCNotifyConfig {
        ignore_path_monitor: conf.get("ignore_path_monitor").unwrap(),
        data_path: conf.get("data_path").unwrap(),
        commands_to_run: conf.get("commands_to_run").unwrap(),
        notify_interval: conf.get("notify_interval").unwrap(),
    };
}


fn main() {
    let mut config = load_config();
    println!("Load Config: {:?}", config);

    config.ignore_path_monitor.push("files_external".to_string());
    config.ignore_path_monitor.push("files_trashbin".to_string());
    config.ignore_path_monitor.push("files_versions".to_string());
    config.ignore_path_monitor.push("nextcloud.log".to_string());

    let data_path = format!("{}/",config.data_path.clone());
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, Error>| {
            match res {
               Ok(event) => {
                   for path in event.paths {
                       let sliced_path = path.to_str().unwrap().to_string().replace(&data_path, "");
                       for ignore_path in config.ignore_path_monitor.clone() {
                           if sliced_path.starts_with(&ignore_path) || sliced_path.starts_with("appdata_oc") {
                               return;
                           }
                       }

                       (|changed_files: &mut Vec<String>, path: &String| {
                           let is_none = changed_files.iter().position(|x| x == path).is_none();
                           if is_none {
                               changed_files.push(path.to_string());
                           }
                       })(&mut CHANGED_FILES.lock().unwrap(), &path.to_str().unwrap().to_string());
                   }
               },
               Err(e) => println!("watch error: {:?}", e),
            }
        }).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(&config.data_path), RecursiveMode::Recursive).unwrap();

    loop {
        thread::sleep(time::Duration::from_secs(config.notify_interval));
        (|changed_files: &mut Vec<String>| {
            while changed_files.len() > 0 {
                let file = changed_files.pop().unwrap().replace(&config.data_path, "");
                println!("New Changes: {}", file);
                let command = &config.commands_to_run.replace("{file}", &file);
                let command = command.replace("{data_path}", &config.data_path);

                Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .spawn()
                    .expect("Failed to execute command");
            }
        })(&mut CHANGED_FILES.lock().unwrap())
    }
}
