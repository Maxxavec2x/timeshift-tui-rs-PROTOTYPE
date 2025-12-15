use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::fmt;
use std::process::Command;
use std::str;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct Device {
    num: u8,
    device_name: String,
    size: usize,
    device_type: String, //should be an enum, will do it later
    label: String, // I legit don't know what that is, mine is always left blank on my system, and I
                   // do not find the documentation (tell me if you know what that is)
}

impl Device {
    pub fn new(
        num: u8,
        device_name: String,
        size: usize,
        device_type: String,
        label: String,
    ) -> Self {
        Device {
            num,
            device_name,
            size,
            device_type,
            label,
        }
    }
}
impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {}",
            self.num, self.device_name, self.size, self.device_type, self.label
        )
    }
}

#[derive(Debug, Default, Clone)] // We cannot use the copy trait because there is a String in our
// struct (:/)
pub struct Snapshot {
    num: u8,

    date: NaiveDateTime,

    tags: char,
    description: String,
}

impl Snapshot {
    pub fn new(num: u8, name: &str, tags: char, description: String) -> Self {
        let date = NaiveDateTime::parse_from_str(name, "%Y-%m-%d_%H-%M-%S")
            .expect("Could not parse a date from the name of the snapshot");
        Snapshot {
            num,
            date,
            tags,
            description,
        }
    }
}
impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {}",
            self.num,
            self.date.format("%Y-%m-%d %H:%M:%S"),
            self.tags,
            self.description
        )
    }
}

#[derive(Debug, Default)]
pub struct Timeshift {
    //I just figured out there can be multiple devices, so now we have an
    //hashmap with device:snapshot[]
    pub devices_map: HashMap<Device, Vec<Snapshot>>,
    pub devices_map_by_name: HashMap<String, Vec<Snapshot>>,
}

impl Timeshift {
    pub fn new() -> Self {
        let mut devices_map: HashMap<Device, Vec<Snapshot>> = HashMap::new();
        let mut devices_map_by_name: HashMap<String, Vec<Snapshot>> = HashMap::new();
        let devices: Vec<Device> = Self::get_devices();
        for device in devices {
            devices_map.insert(device.clone(), Self::get_snapshots(device.clone()));
            devices_map_by_name.insert(device.clone().device_name, Self::get_snapshots(device));
            // Le fait de devoir faire des device.clone parce que je suis un noob avec les lifetime
            // est cursed, mais je réglerais ça + tard
            // // HAHHAHAHAH C'EST LA GUERRE DES CLONES (vous l'avez ?)
        }

        Timeshift {
            devices_map,
            devices_map_by_name,
        }
    }

    pub fn get_current_snapshot(&self, num: usize) -> Snapshot {
        todo!()
        // It would be better  to use a reference to the currnt
        // snapshot, but since im a noob i will just use the clone
        // trait (snapshot uses String)
        // : https://users.rust-lang.org/t/rust-noob-asks-lifetime-of-a-reference-in-a-struct/47808/3
    }

    pub fn get_snapshots(device: Device) -> Vec<Snapshot> {
        let mut result: Vec<Snapshot> = Vec::new();
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list")
            .arg("--snapshot-device")
            .arg(device.device_name)
            .output()
            .expect("Couldn't get snapshots list");
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(10) {
            //getto, à améliorer
            if line.is_empty() {
                break;
            }
            result.push(Self::parse_snapshot_output(line));
        }
        result
    }

    pub fn parse_snapshot_output(output: &str) -> Snapshot {
        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() < 5 {
            // panic!("Invalid snapshot format");
        }
        let num = parts[0].parse::<u8>().expect("Could not parse num");
        let name = parts[2];
        let tags = parts[3].parse::<char>().expect("could not parse Tags");
        let description = parts[4..].join(" ").to_string();
        // On récupère tout le
        // reste (heuresement que
        // le truc est à la fin)

        Snapshot::new(num, name, tags, description)
    }

    pub fn get_devices() -> Vec<Device> {
        let mut result: Vec<Device> = Vec::new();
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list-device")
            .output()
            .expect("Couldn't get device list");
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(7) {
            //getto, à améliorer
            if line.is_empty() {
                break;
            }
            result.push(Self::parse_device_output(line));
        }
        result
    }

    pub fn parse_device_output(output: &str) -> Device {
        let parts: Vec<&str> = output.split_whitespace().collect();
        let num = parts[0].parse::<u8>().expect("Could not parse num");
        let device_name = parts[2].to_string();
        let size = parts[3].parse::<usize>().expect("Could not parse size");
        let device_type = parts[4].to_string();
        let label = parts[6].to_string();
        // On récupère tout le
        // reste (heuresement que
        // le truc est à la fin)

        Device::new(num, device_name, size, device_type, label)
    }
}
